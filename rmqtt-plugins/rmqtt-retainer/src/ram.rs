use crate::{PluginConfig, ERR_NOT_SUPPORTED};
use once_cell::sync::OnceCell;
use rmqtt::{async_trait::async_trait, log, once_cell, tokio::sync::RwLock};
use rmqtt::{
    broker::{
        default::DefaultRetainStorage,
        types::{Retain, TopicFilter, TopicName},
        RetainStorage,
    },
    Result,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct RamRetainer {
    pub(crate) inner: &'static DefaultRetainStorage,
    cfg: Arc<RwLock<PluginConfig>>,
    retain_enable: Arc<AtomicBool>,
}

impl RamRetainer {
    #[inline]
    pub(crate) fn get_or_init(
        cfg: Arc<RwLock<PluginConfig>>,
        retain_enable: Arc<AtomicBool>,
    ) -> &'static RamRetainer {
        static INSTANCE: OnceCell<RamRetainer> = OnceCell::new();
        INSTANCE.get_or_init(|| Self { inner: DefaultRetainStorage::instance(), cfg, retain_enable })
    }

    #[inline]
    pub(crate) async fn remove_expired_messages(&self) -> usize {
        self.inner.remove_expired_messages().await
    }
}

#[async_trait]
impl RetainStorage for &'static RamRetainer {
    ///topic - concrete topic
    async fn set(&self, topic: &TopicName, retain: Retain, expiry_interval: Option<Duration>) -> Result<()> {
        if !self.retain_enable.load(Ordering::SeqCst) {
            log::error!("{}", ERR_NOT_SUPPORTED);
            return Ok(());
        }

        let (max_retained_messages, max_payload_size) = {
            let cfg = self.cfg.read().await;
            (cfg.max_retained_messages, *cfg.max_payload_size)
        };

        if retain.publish.payload.len() > max_payload_size {
            log::warn!("Retain message payload exceeding limit, topic: {:?}, retain: {:?}", topic, retain);
            return Ok(());
        }

        if max_retained_messages > 0 && self.inner.count().await >= max_retained_messages {
            log::warn!(
                "The retained message has exceeded the maximum limit of: {}, topic: {:?}, retain: {:?}",
                max_retained_messages,
                topic,
                retain
            );
            return Ok(());
        }

        self.inner.set_with_timeout(topic, retain, expiry_interval).await
    }

    ///topic_filter - Topic filter
    async fn get(&self, topic_filter: &TopicFilter) -> Result<Vec<(TopicName, Retain)>> {
        if !self.retain_enable.load(Ordering::SeqCst) {
            log::error!("{}", ERR_NOT_SUPPORTED);
            Ok(Vec::new())
        } else {
            Ok(self.inner.get_message(topic_filter).await?)
        }
    }

    #[inline]
    async fn count(&self) -> isize {
        self.inner.count().await
    }

    #[inline]
    async fn max(&self) -> isize {
        self.inner.max().await
    }
}
