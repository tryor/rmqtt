use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use futures::FutureExt;

use client::NodeGrpcClient;

use crate::{Addr, ClientId, Result};
use crate::broker::{ClearSubscriptions, SubRelations, SubRelationsMap};
use crate::broker::session::SessionOfflineInfo;
use crate::broker::types::{
    From, Id, IsAdmin, NodeId, Publish, Retain, Route, SessionStatus, SubsSearchParams, SubsSearchResult,
    TopicFilter, TopicName,
};

pub mod client;
pub mod server;

#[allow(dead_code)]
pub(crate) mod pb {
    include!(concat!(env!("OUT_DIR"), "/pb.rs"));
}

///Reserved within 1000
pub type MessageType = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Forwards(From, Publish),
    ForwardsTo(From, Publish, SubRelations),
    Kick(Id, ClearSubscriptions, IsAdmin),
    GetRetains(TopicFilter),
    SubscriptionsSearch(SubsSearchParams),
    SubscriptionsGet(ClientId),
    RoutesGet(usize),
    RoutesGetBy(TopicFilter),
    NumberOfClients,
    NumberOfSessions,
    Online(ClientId),
    SessionStatus(ClientId),
    Data(Vec<u8>),
}

impl Message {
    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self).map_err(anyhow::Error::new)?)
    }
    #[inline]
    pub fn decode(data: &[u8]) -> Result<Message> {
        Ok(bincode::deserialize::<Message>(data).map_err(anyhow::Error::new)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageReply {
    Success,
    Forwards(SubRelationsMap),
    Error(String),
    Kick(Option<SessionOfflineInfo>),
    GetRetains(Vec<(TopicName, Retain)>),
    SubscriptionsSearch(Vec<SubsSearchResult>),
    SubscriptionsGet(Option<Vec<SubsSearchResult>>),
    RoutesGet(Vec<Route>),
    RoutesGetBy(Vec<Route>),
    NumberOfClients(usize),
    NumberOfSessions(usize),
    Online(bool),
    SessionStatus(Option<SessionStatus>),
    Data(Vec<u8>),
}

impl MessageReply {
    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self).map_err(anyhow::Error::new)?)
    }
    #[inline]
    pub fn decode(data: &[u8]) -> Result<MessageReply> {
        Ok(bincode::deserialize::<MessageReply>(data).map_err(anyhow::Error::new)?)
    }
}

pub struct MessageSender {
    client: NodeGrpcClient,
    msg_type: MessageType,
    msg: Message,
}

impl MessageSender {
    #[inline]
    pub fn new(client: NodeGrpcClient, msg_type: MessageType, msg: Message) -> Self {
        Self { client, msg_type, msg }
    }

    #[inline]
    pub async fn send(self) -> Result<MessageReply> {
        match self.client.send_message(self.msg_type, self.msg).await {
            Ok(reply) => Ok(reply),
            Err(e) => {
                log::warn!("error sending message, {:?}", e);
                Err(e)
            }
        }
    }
}

pub type GrpcClients = Arc<HashMap<NodeId, (Addr, NodeGrpcClient), ahash::RandomState>>;

pub struct MessageBroadcaster {
    grpc_clients: GrpcClients,
    msg_type: MessageType,
    msg: Message,
}

impl MessageBroadcaster {
    #[inline]
    pub fn new(grpc_clients: GrpcClients, msg_type: MessageType, msg: Message) -> Self {
        assert!(!grpc_clients.is_empty(), "gRPC clients is empty!");
        Self { grpc_clients, msg_type, msg }
    }

    #[inline]
    pub async fn join_all(self) -> Vec<(NodeId, Result<MessageReply>)> {
        let msg = self.msg;
        let mut senders = Vec::new();
        for (id, (_, grpc_client)) in self.grpc_clients.iter() {
            let msg_type = self.msg_type;
            let msg = msg.clone();
            let fut = async move { (*id, grpc_client.send_message(msg_type, msg).await) };
            senders.push(fut.boxed());
        }
        futures::future::join_all(senders).await
    }

    #[inline]
    pub async fn select_ok<R, F>(self, check_fn: F) -> Result<R>
        where
            R: std::any::Any + Send + Sync,
            F: Fn(MessageReply) -> Result<R> + Send + Sync,
    {
        let msg = self.msg;
        let mut senders = Vec::new();
        let max_idx = self.grpc_clients.len() - 1;
        for (i, (_, (_, grpc_client))) in self.grpc_clients.iter().enumerate() {
            if i == max_idx {
                senders.push(Self::send(grpc_client, self.msg_type, msg, &check_fn).boxed());
                break;
            } else {
                senders.push(Self::send(grpc_client, self.msg_type, msg.clone(), &check_fn).boxed());
            }
        }
        let (reply, _) = futures::future::select_ok(senders).await?;
        Ok(reply)
    }

    #[inline]
    async fn send<R, F>(
        grpc_client: &NodeGrpcClient,
        typ: MessageType,
        msg: Message,
        check_fn: &F,
    ) -> Result<R>
        where
            R: std::any::Any + Send + Sync,
            F: Fn(MessageReply) -> Result<R> + Send + Sync,
    {
        match grpc_client.send_message(typ, msg).await {
            Ok(r) => {
                log::debug!("OK reply: {:?}", r);
                check_fn(r)
            }
            Err(e) => {
                log::debug!("ERROR reply: {:?}", e);
                Err(e)
            }
        }
    }
}
