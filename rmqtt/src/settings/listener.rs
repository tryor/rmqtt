use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use serde::de::{self, Deserialize, Deserializer};

use crate::broker::types::QoS;

use super::{Bytesize, deserialize_addr, deserialize_duration, to_duration};

type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;

type Port = u16;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Listeners {
    #[serde(rename = "tcp")]
    #[serde(default)]
    _tcps: HashMap<String, ListenerInner>,

    #[serde(rename = "tls")]
    #[serde(default)]
    _tlss: HashMap<String, ListenerInner>,

    #[serde(default, skip)]
    pub tcps: HashMap<Port, Listener>,
    #[serde(default, skip)]
    pub tlss: HashMap<Port, Listener>,
}

impl Listeners {
    #[inline]
    pub(crate) fn init(&mut self) {
        for (name, mut inner) in self._tcps.drain() {
            inner.name = name;
            self.tcps.insert(inner.addr.port(), Listener::new(inner));
        }

        for (name, mut inner) in self._tlss.drain() {
            inner.name = name;
            self.tlss.insert(inner.addr.port(), Listener::new(inner));
        }
    }

    #[inline]
    pub fn tcp(&self, port: u16) -> Option<Listener> {
        self.tcps.get(&port).cloned()
    }

    #[inline]
    pub fn tls(&self, port: u16) -> Option<Listener> {
        self.tlss.get(&port).cloned()
    }

    #[inline]
    pub fn get(&self, port: u16) -> Option<Listener> {
        if let Some(tcp) = self.tcp(port) {
            return Some(tcp);
        }
        self.tls(port)
    }

    #[inline]
    pub(crate) fn set_default(&mut self) {
        let inner = Listener::default();
        self.tcps.insert(inner.addr.port(), inner);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Listener {
    inner: Arc<ListenerInner>,
}

impl Listener {
    #[inline]
    fn new(inner: ListenerInner) -> Self {
        Self { inner: Arc::new(inner) }
    }
}

impl Deref for Listener {
    type Target = ListenerInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListenerInner {
    #[serde(default)]
    pub name: String,
    #[serde(default = "ListenerInner::addr_default", deserialize_with = "deserialize_addr")]
    pub addr: SocketAddr,
    #[serde(default = "ListenerInner::workers_default")]
    pub workers: usize,
    #[serde(default = "ListenerInner::max_connections_default")]
    pub max_connections: usize,
    #[serde(default = "ListenerInner::max_handshaking_limit_default")]
    pub max_handshaking_limit: usize,
    #[serde(default = "ListenerInner::max_packet_size_default")]
    pub max_packet_size: Bytesize,
    #[serde(default = "ListenerInner::backlog_default")]
    pub backlog: i32,
    #[serde(default = "ListenerInner::idle_timeout_default", deserialize_with = "deserialize_duration")]
    pub idle_timeout: Duration,
    #[serde(default = "ListenerInner::allow_anonymous_default")]
    pub allow_anonymous: bool,
    #[serde(
    default = "ListenerInner::min_keepalive_default",
    //deserialize_with = "deserialize_duration"
    )]
    pub min_keepalive: u16,
    #[serde(default = "ListenerInner::keepalive_backoff_default")]
    pub keepalive_backoff: f32,
    #[serde(default = "ListenerInner::max_inflight_default")]
    pub max_inflight: usize,
    #[serde(default = "ListenerInner::handshake_timeout_default", deserialize_with = "deserialize_duration")]
    pub handshake_timeout: Duration,
    #[serde(default = "ListenerInner::max_mqueue_len_default")]
    pub max_mqueue_len: usize,
    #[serde(
    default = "ListenerInner::mqueue_rate_limit_default",
    deserialize_with = "ListenerInner::deserialize_mqueue_rate_limit"
    )]
    pub mqueue_rate_limit: (NonZeroU32, Duration),

    #[serde(default = "ListenerInner::max_clientid_len_default")]
    pub max_clientid_len: usize,

    #[serde(
    default = "ListenerInner::max_qos_allowed_default",
    deserialize_with = "ListenerInner::deserialize_max_qos_allowed"
    )]
    pub max_qos_allowed: QoS,

    #[serde(default = "ListenerInner::max_topic_levels_default")]
    pub max_topic_levels: usize,

    #[serde(default = "ListenerInner::retain_available_default")]
    pub retain_available: bool,
    #[serde(
    default = "ListenerInner::session_expiry_interval_default",
    deserialize_with = "deserialize_duration"
    )]
    pub session_expiry_interval: Duration,

    #[serde(
    default = "ListenerInner::message_retry_interval_default",
    deserialize_with = "deserialize_duration"
    )]
    pub message_retry_interval: Duration,

    #[serde(
    default = "ListenerInner::message_expiry_interval_default",
    deserialize_with = "deserialize_duration"
    )]
    pub message_expiry_interval: Duration,

    #[serde(default = "ListenerInner::max_awaiting_rel_default")]
    pub max_awaiting_rel: usize,
    #[serde(default = "ListenerInner::await_rel_timeout_default", deserialize_with = "deserialize_duration")]
    pub await_rel_timeout: Duration,

    #[serde(default = "ListenerInner::max_subscriptions_default")]
    pub max_subscriptions: usize,

    #[serde(default = "ListenerInner::shared_subscription_default")]
    pub shared_subscription: bool,

    pub cert: Option<String>,
    pub key: Option<String>,
}

impl Default for ListenerInner {
    fn default() -> Self {
        Self {
            name: "external".into(),
            addr: ListenerInner::addr_default(),
            workers: ListenerInner::workers_default(),
            max_connections: ListenerInner::max_connections_default(),
            max_handshaking_limit: ListenerInner::max_handshaking_limit_default(),
            max_packet_size: ListenerInner::max_packet_size_default(),
            backlog: ListenerInner::backlog_default(),
            idle_timeout: ListenerInner::idle_timeout_default(),
            allow_anonymous: ListenerInner::allow_anonymous_default(),
            min_keepalive: ListenerInner::min_keepalive_default(),
            keepalive_backoff: ListenerInner::keepalive_backoff_default(),
            max_inflight: ListenerInner::max_inflight_default(),
            handshake_timeout: ListenerInner::handshake_timeout_default(),
            max_mqueue_len: ListenerInner::max_mqueue_len_default(),
            mqueue_rate_limit: ListenerInner::mqueue_rate_limit_default(),
            max_clientid_len: ListenerInner::max_clientid_len_default(),
            max_qos_allowed: ListenerInner::max_qos_allowed_default(),
            max_topic_levels: ListenerInner::max_topic_levels_default(),
            retain_available: ListenerInner::retain_available_default(),
            session_expiry_interval: ListenerInner::session_expiry_interval_default(),
            message_retry_interval: ListenerInner::message_retry_interval_default(),
            message_expiry_interval: ListenerInner::message_expiry_interval_default(),
            max_awaiting_rel: ListenerInner::max_awaiting_rel_default(),
            await_rel_timeout: ListenerInner::await_rel_timeout_default(),
            max_subscriptions: ListenerInner::max_subscriptions_default(),
            shared_subscription: ListenerInner::shared_subscription_default(),
            cert: None,
            key: None,
        }
    }
}

impl ListenerInner {
    #[inline]
    fn addr_default() -> SocketAddr {
        ([0, 0, 0, 0], 1883).into()
    }
    #[inline]
    fn workers_default() -> usize {
        8
    }
    #[inline]
    fn max_connections_default() -> usize {
        1024000
    }
    #[inline]
    fn max_handshaking_limit_default() -> usize {
        500
    }
    #[inline]
    fn max_packet_size_default() -> Bytesize {
        Bytesize(1024 * 1024)
    }
    #[inline]
    fn backlog_default() -> i32 {
        1024
    }
    #[inline]
    fn idle_timeout_default() -> Duration {
        Duration::from_secs(15)
    }
    #[inline]
    fn allow_anonymous_default() -> bool {
        true
    }
    #[inline]
    fn min_keepalive_default() -> u16 {
        0
    }
    #[inline]
    fn keepalive_backoff_default() -> f32 {
        0.75
    }
    #[inline]
    fn max_inflight_default() -> usize {
        16
    }
    #[inline]
    fn handshake_timeout_default() -> Duration {
        Duration::from_secs(15)
    }
    #[inline]
    fn max_mqueue_len_default() -> usize {
        1000
    }
    #[inline]
    fn mqueue_rate_limit_default() -> (NonZeroU32, Duration) {
        (NonZeroU32::new(u32::max_value()).unwrap(), Duration::from_secs(1))
    }
    #[inline]
    fn max_clientid_len_default() -> usize {
        65535
    }
    #[inline]
    fn max_qos_allowed_default() -> QoS {
        QoS::ExactlyOnce
    }
    #[inline]
    fn max_topic_levels_default() -> usize {
        0
    }

    #[inline]
    fn retain_available_default() -> bool {
        true
    }
    #[inline]
    fn session_expiry_interval_default() -> Duration {
        Duration::from_secs(7200)
    }
    #[inline]
    fn message_retry_interval_default() -> Duration {
        Duration::from_secs(30)
    }
    #[inline]
    fn message_expiry_interval_default() -> Duration {
        Duration::from_secs(30)
    }
    #[inline]
    fn max_awaiting_rel_default() -> usize {
        100
    }
    #[inline]
    fn await_rel_timeout_default() -> Duration {
        Duration::from_secs(300)
    }
    #[inline]
    fn max_subscriptions_default() -> usize {
        0
    }
    #[inline]
    fn shared_subscription_default() -> bool {
        true
    }

    #[inline]
    pub fn handshake_timeout(&self) -> u16 {
        let millis = self.handshake_timeout.as_millis();
        if millis > 0xffff {
            0xffff
        } else {
            millis as u16
        }
    }

    #[inline]
    fn deserialize_mqueue_rate_limit<'de, D>(deserializer: D) -> Result<(NonZeroU32, Duration), D::Error>
        where
            D: Deserializer<'de>,
    {
        let v = String::deserialize(deserializer)?;
        let pair: Vec<&str> = v.split(',').collect();
        if pair.len() == 2 {
            let burst = NonZeroU32::from_str(pair[0])
                .map_err(|e| de::Error::custom(format!("mqueue_rate_limit, burst format error, {:?}", e)))?;
            let replenish_n_per = to_duration(pair[1]);
            if replenish_n_per.as_millis() == 0 {
                return Err(de::Error::custom(format!(
                    "mqueue_rate_limit, value format error, {}",
                    pair.join(",")
                )));
            }
            Ok((burst, replenish_n_per))
        } else {
            Err(de::Error::custom(format!("mqueue_rate_limit, value format error, {}", pair.join(","))))
        }
    }
    #[inline]
    fn deserialize_max_qos_allowed<'de, D>(deserializer: D) -> Result<QoS, D::Error>
        where
            D: Deserializer<'de>,
    {
        let qos = match u8::deserialize(deserializer)? {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => return Err(de::Error::custom("QoS configuration error, only values (0,1,2) are supported")),
        };
        Ok(qos)
    }
}
