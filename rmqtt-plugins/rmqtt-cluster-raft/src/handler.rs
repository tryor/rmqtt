use rmqtt_raft::Mailbox;

use rmqtt::{async_trait::async_trait, log, MqttError, tokio};
use rmqtt::{
    broker::hook::{Handler, HookResult, Parameter, ReturnType},
    grpc::{Message as GrpcMessage, MessageReply},
    Id, Runtime,
};
use rmqtt::rust_box::task_executor::SpawnExt;
use rmqtt::broker::Shared;

use super::{executor, hook_message_dropped, retainer::ClusterRetainer, shared::ClusterShared};
use super::config::{BACKOFF_STRATEGY, retry};
use super::message::Message;

pub(crate) struct HookHandler {
    shared: &'static ClusterShared,
    retainer: &'static ClusterRetainer,
    raft_mailbox: Mailbox,
}

impl HookHandler {
    pub(crate) fn new(
        shared: &'static ClusterShared,
        retainer: &'static ClusterRetainer,
        raft_mailbox: Mailbox,
    ) -> Self {
        Self { shared, retainer, raft_mailbox }
    }
}

#[async_trait]
impl Handler for HookHandler {
    async fn hook(&self, param: &Parameter, acc: Option<HookResult>) -> ReturnType {
        log::debug!("hook, Parameter type: {:?}", param.get_type());
        match param {
            Parameter::ClientDisconnected(_s, c, r) => {
                log::debug!("{:?} hook::ClientDisconnected reason: {:?}", c.id, r);
                if !r.contains("Kicked") {
                    let msg = Message::Disconnected { id: c.id.clone() }.encode().unwrap();
                    let raft_mailbox = self.raft_mailbox.clone();
                    tokio::spawn(async move {
                        if let Err(e) = retry(BACKOFF_STRATEGY.clone(), || async {
                            let msg = msg.clone();
                            let mailbox = raft_mailbox.clone();
                            let res = async move { mailbox.send(msg).await }
                                .spawn(executor()).result().await
                                .map_err(|_| MqttError::from("Handler::hook(Message::Disconnected), task execution failure"))?
                                .map_err(|e| MqttError::from(e.to_string()))?;
                            Ok(res)
                        })
                            .await
                        {
                            log::warn!(
                                "HookHandler, Message::Disconnected, raft mailbox send error, {:?}",
                                e
                            );
                        }
                    });
                }
            }

            Parameter::SessionTerminated(_s, c, _r) => {
                let msg = Message::SessionTerminated { id: c.id.clone() }.encode().unwrap();
                let raft_mailbox = self.raft_mailbox.clone();
                tokio::spawn(async move {
                    if let Err(e) = retry(BACKOFF_STRATEGY.clone(), || async {
                        let msg = msg.clone();
                        let mailbox = raft_mailbox.clone();
                        let res = async move { mailbox.send(msg).await }
                            .spawn(executor()).result().await
                            .map_err(|_| MqttError::from("Handler::hook(Message::SessionTerminated), task execution failure"))?
                            .map_err(|e| MqttError::from(e.to_string()))?;
                        Ok(res)
                    })
                        .await
                    {
                        log::warn!(
                            "HookHandler, Message::SessionTerminated, raft mailbox send error, {:?}",
                            e
                        );
                    }
                });
            }

            Parameter::GrpcMessageReceived(typ, msg) => {
                log::debug!("GrpcMessageReceived, type: {}, msg: {:?}", typ, msg);
                if self.shared.message_type != *typ {
                    return (true, acc);
                }
                match msg {
                    GrpcMessage::ForwardsTo(from, publish, sub_rels) => {
                        if let Err(droppeds) =
                        self.shared.forwards_to(from.clone(), publish, sub_rels.clone()).await
                        {
                            hook_message_dropped(droppeds).await;
                        }
                        return (false, acc);
                    }
                    GrpcMessage::Kick(id, clear_subscriptions, is_admin) => {
                        let mut entry = self.shared.inner().entry(id.clone());
                        let new_acc = match entry.kick(*clear_subscriptions, *is_admin).await {
                            Ok(Some(o)) => {
                                if *is_admin {
                                    self.shared.router().remove_client_status(&id.client_id);
                                }
                                HookResult::GrpcMessageReply(Ok(MessageReply::Kick(Some(o))))
                            }
                            Ok(None) => {
                                self.shared.router().remove_client_status(&id.client_id);
                                HookResult::GrpcMessageReply(Ok(MessageReply::Kick(None)))
                            }
                            Err(e) => HookResult::GrpcMessageReply(Err(e)),
                        };
                        return (false, Some(new_acc));
                    }
                    GrpcMessage::GetRetains(topic_filter) => {
                        log::debug!("[GrpcMessage::GetRetains] topic_filter: {:?}", topic_filter);
                        let new_acc = match self.retainer.inner().get(topic_filter).await {
                            Ok(retains) => {
                                HookResult::GrpcMessageReply(Ok(MessageReply::GetRetains(retains)))
                            }
                            Err(e) => HookResult::GrpcMessageReply(Err(e)),
                        };
                        return (false, Some(new_acc));
                    }
                    GrpcMessage::SubscriptionsGet(clientid) => {
                        let id = Id::from(Runtime::instance().node.id(), clientid.clone());
                        let entry = self.shared.inner().entry(id);
                        let new_acc = HookResult::GrpcMessageReply(Ok(MessageReply::SubscriptionsGet(
                            entry.subscriptions().await,
                        )));
                        return (false, Some(new_acc));
                    }
                    _ => {
                        log::error!("unimplemented, {:?}", param)
                    }
                }
            }
            _ => {
                log::error!("unimplemented, {:?}", param)
            }
        }
        (true, acc)
    }
}

// async fn forwards(from: From, publish: Publish) -> SharedSubRelations {
//     match Runtime::instance().extends.shared().await.forwards_and_get_shareds(from, publish).await {
//         Err(droppeds) => {
//             hook_message_dropped(droppeds).await;
//             SharedSubRelations::default()
//         }
//         Ok(shared_subs) => shared_subs,
//     }
// }
