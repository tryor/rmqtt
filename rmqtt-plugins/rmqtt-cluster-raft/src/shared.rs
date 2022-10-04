use std::time::Duration;

use futures::future::FutureExt;
use once_cell::sync::OnceCell;

use rmqtt::{anyhow, async_trait::async_trait, futures, log, once_cell, tokio};
use rmqtt::{
    broker::{
        default::DefaultShared,
        Entry,
        session::{ClientInfo, Session, SessionOfflineInfo},
        Shared, SubRelations, SubRelationsMap, types::{
            From, Id, IsAdmin, NodeId, Publish, Reason, SessionStatus, Subscribe, SubscribeReturn,
            SubsSearchParams, SubsSearchResult, To, Tx, Unsubscribe,
        },
    },
    grpc::{Message, MessageReply, MessageType},
    MqttError, Result, Runtime,
};
use rmqtt::broker::Router;

use super::{ClusterRouter, GrpcClients, MessageSender, NodeGrpcClient};
use super::message::{get_client_node_id, Message as RaftMessage, MessageReply as RaftMessageReply};

pub struct ClusterLockEntry {
    inner: Box<dyn Entry>,
    cluster_shared: &'static ClusterShared,
    prev_node_id: Option<NodeId>,
}

impl ClusterLockEntry {
    #[inline]
    pub fn new(
        inner: Box<dyn Entry>,
        cluster_shared: &'static ClusterShared,
        prev_node_id: Option<NodeId>,
    ) -> Self {
        Self { inner, cluster_shared, prev_node_id }
    }
}

#[async_trait]
impl Entry for ClusterLockEntry {
    #[inline]
    async fn try_lock(&self) -> Result<Box<dyn Entry>> {
        let msg = RaftMessage::HandshakeTryLock { id: self.id() }.encode()?;
        let raft_mailbox = self.cluster_shared.router.raft_mailbox().await;
        let reply = raft_mailbox.send(msg).await.map_err(anyhow::Error::new)?;
        let mut prev_node_id = None;
        if !reply.is_empty() {
            match RaftMessageReply::decode(&reply)? {
                RaftMessageReply::Error(e) => {
                    return Err(MqttError::Msg(e));
                }
                RaftMessageReply::HandshakeTryLock(prev_id) => {
                    prev_node_id = prev_id.map(|id| id.node_id);
                    log::debug!(
                        "{:?} ClusterLockEntry try_lock prev_node_id: {:?}",
                        self.client().map(|c| c.id.clone()),
                        prev_node_id
                    );
                } // _ => unreachable!()
            }
        }
        Ok(Box::new(ClusterLockEntry::new(self.inner.try_lock().await?, self.cluster_shared, prev_node_id)))
    }

    #[inline]
    fn id(&self) -> Id {
        self.inner.id()
    }

    #[inline]
    fn id_same(&self) -> Option<bool> {
        self.inner.id_same()
    }

    #[inline]
    fn exist(&self) -> bool {
        self.inner.exist()
    }

    #[inline]
    async fn set(&mut self, session: Session, tx: Tx, conn: ClientInfo) -> Result<()> {
        let msg = RaftMessage::Connected { id: session.id.clone() }.encode()?;
        let raft_mailbox = self.cluster_shared.router.raft_mailbox().await;
        let reply = raft_mailbox.send(msg).await.map_err(anyhow::Error::new)?;
        if !reply.is_empty() {
            let reply = RaftMessageReply::decode(&reply)?;
            match reply {
                RaftMessageReply::Error(e) => {
                    return Err(MqttError::Msg(e));
                }
                _ => {
                    log::error!("unreachable!(), {:?}", reply);
                    unreachable!()
                }
            }
        }
        self.inner.set(session, tx, conn).await
    }

    #[inline]
    async fn remove(&mut self) -> Result<Option<(Session, Tx, ClientInfo)>> {
        self.inner.remove().await
    }

    #[inline]
    async fn remove_with(&mut self, id: &Id) -> Result<Option<(Session, Tx, ClientInfo)>> {
        self.inner.remove_with(id).await
    }

    #[inline]
    async fn kick(
        &mut self,
        clear_subscriptions: bool,
        is_admin: IsAdmin,
    ) -> Result<Option<SessionOfflineInfo>> {
        log::debug!(
            "{:?} ClusterLockEntry kick ..., clear_subscriptions: {}, is_admin: {}",
            self.client().map(|c| c.id.clone()),
            clear_subscriptions,
            is_admin
        );
        let id = self.id();

        let prev_node_id = if is_admin {
            let raft_mailbox = self.cluster_shared.router.raft_mailbox().await;
            let node_id = get_client_node_id(raft_mailbox, &id.client_id).await?;
            node_id.unwrap_or(id.node_id)
        } else {
            self.prev_node_id.unwrap_or(id.node_id)
        };
        log::debug!("{:?} kick, prev_node_id: {:?}, is_admin: {}", id, self.prev_node_id, is_admin);
        if prev_node_id == id.node_id {
            //kicked from local
            self.inner.kick(clear_subscriptions, is_admin).await
        } else {
            //kicked from other node
            if let Some(client) = self.cluster_shared.grpc_client(prev_node_id) {
                let mut msg_sender = MessageSender {
                    client,
                    msg_type: self.cluster_shared.message_type,
                    msg: Message::Kick(id.clone(), true, is_admin), //clear_subscriptions
                    max_retries: 0,
                    retry_interval: Duration::from_millis(500),
                };
                match msg_sender.send().await {
                    Ok(reply) => {
                        if let MessageReply::Kick(Some(kicked)) = reply {
                            log::debug!("{:?} kicked: {:?}", id, kicked);
                            Ok(Some(kicked))
                        } else {
                            log::info!(
                                "{:?} Message::Kick from other node, prev_node_id: {:?}, reply: {:?}",
                                id,
                                prev_node_id,
                                reply
                            );
                            Ok(None)
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "{:?} Message::Kick from other node, prev_node_id: {:?}, error: {:?}",
                            id,
                            prev_node_id,
                            e
                        );
                        Ok(None)
                    }
                }
            } else {
                return Err(MqttError::Msg(format!(
                    "kick error, grpc_client is not exist, prev_node_id: {:?}",
                    prev_node_id
                )));
            }
        }
    }

    #[inline]
    async fn online(&self) -> bool {
        self.cluster_shared.router.is_online(0, &self.id().client_id).await
    }

    #[inline]
    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    #[inline]
    fn session(&self) -> Option<Session> {
        self.inner.session()
    }

    #[inline]
    fn client(&self) -> Option<ClientInfo> {
        self.inner.client()
    }

    #[inline]
    fn tx(&self) -> Option<Tx> {
        self.inner.tx()
    }

    #[inline]
    async fn subscribe(&self, subscribe: &Subscribe) -> Result<SubscribeReturn> {
        self.inner.subscribe(subscribe).await
    }

    #[inline]
    async fn unsubscribe(&self, unsubscribe: &Unsubscribe) -> Result<bool> {
        self.inner.unsubscribe(unsubscribe).await
    }

    #[inline]
    async fn publish(&self, from: From, p: Publish) -> Result<(), (From, Publish, Reason)> {
        self.inner.publish(from, p).await
    }

    #[inline]
    async fn subscriptions(&self) -> Option<Vec<SubsSearchResult>> {
        let id = self.cluster_shared.router.id(&self.id().client_id)?;
        if id.node_id == Runtime::instance().node.id() {
            self.inner.subscriptions().await
        } else {
            //from other node
            if let Some(client) = self.cluster_shared.grpc_client(id.node_id) {
                let reply = MessageSender {
                    client,
                    msg_type: self.cluster_shared.message_type,
                    msg: Message::SubscriptionsGet(id.client_id.clone()),
                    max_retries: 0,
                    retry_interval: Duration::from_millis(500),
                }
                    .send()
                    .await;
                match reply {
                    Ok(MessageReply::SubscriptionsGet(subs)) => subs,
                    Err(e) => {
                        log::warn!("Message::SubscriptionsGet, error: {:?}", e);
                        None
                    }
                    _ => unreachable!(),
                }
            } else {
                log::error!("get subscriptions error, grpc_client is not exist, node_id: {}", id.node_id,);
                None
            }
        }
    }
}

pub struct ClusterShared {
    inner: &'static DefaultShared,
    router: &'static ClusterRouter,
    grpc_clients: GrpcClients,
    pub message_type: MessageType,
}

impl ClusterShared {
    #[inline]
    pub(crate) fn get_or_init(
        router: &'static ClusterRouter,
        grpc_clients: GrpcClients,
        message_type: MessageType,
    ) -> &'static ClusterShared {
        static INSTANCE: OnceCell<ClusterShared> = OnceCell::new();
        INSTANCE.get_or_init(|| Self { inner: DefaultShared::instance(), router, grpc_clients, message_type })
    }

    #[inline]
    pub(crate) fn router(&self) -> &'static ClusterRouter {
        self.router
    }

    #[inline]
    pub(crate) fn inner(&self) -> Box<dyn Shared> {
        Box::new(self.inner)
    }

    #[inline]
    pub(crate) fn grpc_client(&self, node_id: u64) -> Option<NodeGrpcClient> {
        self.grpc_clients.get(&node_id).map(|(_, c)| c.clone())
    }
}

#[async_trait]
impl Shared for &'static ClusterShared {
    #[inline]
    fn entry(&self, id: Id) -> Box<dyn Entry> {
        Box::new(ClusterLockEntry::new(self.inner.entry(id), self, None))
    }

    #[inline]
    fn exist(&self, client_id: &str) -> bool {
        self.inner.exist(client_id)
    }

    #[inline]
    async fn forwards(&self, from: From, publish: Publish) -> Result<(), Vec<(To, From, Publish, Reason)>> {
        log::debug!("[forwards] from: {:?}, publish: {:?}", from, publish);

        let topic = publish.topic();
        let mut relations_map =
            match Runtime::instance().extends.router().await.matches(publish.topic()).await {
                Ok(relations_map) => relations_map,
                Err(e) => {
                    log::warn!("forwards, from:{:?}, topic:{:?}, error: {:?}", from, topic, e);
                    SubRelationsMap::default()
                }
            };

        let mut errs = Vec::new();

        let this_node_id = Runtime::instance().node.id();
        if let Some(relations) = relations_map.remove(&this_node_id) {
            //forwards to local
            if let Err(e) = self.forwards_to(from.clone(), &publish, relations).await {
                errs.extend(e);
            }
        }
        if !relations_map.is_empty() {
            log::debug!("forwards to other nodes, relations_map:{:?}", relations_map);
            //forwards to other nodes
            let mut fut_senders = Vec::new();
            for (node_id, relations) in relations_map {
                if let Some(client) = self.grpc_client(node_id) {
                    let from = from.clone();
                    let publish = publish.clone();
                    let message_type = self.message_type;
                    let fut_sender = async move {
                        let mut msg_sender = MessageSender {
                            client,
                            msg_type: message_type,
                            msg: Message::ForwardsTo(from, publish, relations),
                            max_retries: 1,
                            retry_interval: Duration::from_millis(500),
                        };
                        (node_id, msg_sender.send().await)
                    };
                    fut_senders.push(fut_sender.boxed());
                } else {
                    log::error!(
                        "forwards error, grpc_client is not exist, node_id: {}, relations: {:?}",
                        node_id,
                        relations
                    );
                }
            }

            tokio::spawn(async move {
                let replys = futures::future::join_all(fut_senders).await;
                for (node_id, reply) in replys {
                    if let Err(e) = reply {
                        log::error!(
                            "forwards Message::ForwardsTo to other node, from: {:?}, to: {:?}, error: {:?}",
                            from,
                            node_id,
                            e
                        );
                    }
                }
            });
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    #[inline]
    async fn forwards_to(
        &self,
        from: From,
        publish: &Publish,
        relations: SubRelations,
    ) -> Result<(), Vec<(To, From, Publish, Reason)>> {
        self.inner.forwards_to(from, publish, relations).await
    }

    #[inline]
    async fn forwards_and_get_shareds(
        &self,
        from: From,
        publish: Publish,
    ) -> Result<SubRelationsMap, Vec<(To, From, Publish, Reason)>> {
        self.inner.forwards_and_get_shareds(from, publish).await
    }

    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item=Box<dyn Entry>> + Sync + Send> {
        self.inner.iter()
    }

    #[inline]
    fn random_session(&self) -> Option<(Session, ClientInfo)> {
        self.inner.random_session()
    }

    #[inline]
    async fn session_status(&self, client_id: &str) -> Option<SessionStatus> {
        let try_lock_timeout = self.router.try_lock_timeout;
        self.router.status(client_id).map(|s| SessionStatus {
            handshaking: s.handshaking(try_lock_timeout),
            id: s.id,
            online: s.online,
        })
    }

    #[inline]
    async fn clinet_states_count(&self) -> usize {
        self.router.states_count()
    }

    #[inline]
    fn sessions_count(&self) -> usize {
        self.inner.sessions_count()
    }

    #[inline]
    async fn query_subscriptions(&self, q: SubsSearchParams) -> Vec<SubsSearchResult> {
        self.inner.query_subscriptions(q).await
    }

    #[inline]
    fn get_grpc_clients(&self) -> GrpcClients {
        self.grpc_clients.clone()
    }
}
