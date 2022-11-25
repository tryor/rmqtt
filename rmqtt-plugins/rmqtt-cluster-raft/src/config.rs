use std::time::Duration;

pub(crate) use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
pub(crate) use backoff::future::retry;
use rmqtt_raft::ReadOnlyOption;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::Serializer;
use serde::Serialize;

use rmqtt::{lazy_static, serde_json};
use rmqtt::grpc::MessageType;
use rmqtt::Result;
use rmqtt::settings::{NodeAddr, deserialize_duration, deserialize_duration_option, Options};

lazy_static::lazy_static! {
    pub static ref BACKOFF_STRATEGY: ExponentialBackoff = ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(Duration::from_secs(60)))
        .with_multiplier(2.5).build();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    #[serde(default = "PluginConfig::message_type_default")]
    pub message_type: MessageType,
    pub node_grpc_addrs: Vec<NodeAddr>,
    pub raft_peer_addrs: Vec<NodeAddr>,
    #[serde(default = "PluginConfig::try_lock_timeout_default", deserialize_with = "deserialize_duration")]
    pub try_lock_timeout: Duration, //Message::HandshakeTryLock

    #[serde(default = "PluginConfig::task_exec_queue_workers_default")]
    pub task_exec_queue_workers: usize,
    #[serde(default = "PluginConfig::task_exec_queue_max_default")]
    pub task_exec_queue_max: usize,
    #[serde(default = "PluginConfig::raft_default")]
    pub raft: RaftConfig,
}

impl PluginConfig {
    #[inline]
    pub fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn message_type_default() -> MessageType {
        198
    }

    fn try_lock_timeout_default() -> Duration {
        Duration::from_secs(10)
    }

    fn task_exec_queue_workers_default() -> usize {
        500
    }

    fn task_exec_queue_max_default() -> usize {
        100_000
    }

    fn raft_default() -> RaftConfig {
        RaftConfig { ..Default::default() }
    }

    pub fn merge(&mut self, opts: &Options) {
        if let Some(node_grpc_addrs) = opts.node_grpc_addrs.as_ref() {
            self.node_grpc_addrs = node_grpc_addrs.clone();
        }
        if let Some(raft_peer_addrs) = opts.raft_peer_addrs.as_ref() {
            self.raft_peer_addrs = raft_peer_addrs.clone();
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RaftConfig {
    #[serde(default, deserialize_with = "deserialize_duration_option")]
    pub grpc_timeout: Option<Duration>,
    pub grpc_concurrency_limit: Option<usize>,
    pub grpc_breaker_threshold: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_duration_option")]
    pub grpc_breaker_retry_interval: Option<Duration>,
    pub proposal_batch_size: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_duration_option")]
    pub proposal_batch_timeout: Option<Duration>,
    #[serde(default, deserialize_with = "deserialize_duration_option")]
    pub snapshot_interval: Option<Duration>,
    #[serde(default, deserialize_with = "deserialize_duration_option")]
    pub heartbeat: Option<Duration>,

    /// The number of node.tick invocations that must pass between
    /// elections. That is, if a follower does not receive any message from the
    /// leader of current term before ElectionTick has elapsed, it will become
    /// candidate and start an election. election_tick must be greater than
    /// HeartbeatTick. We suggest election_tick = 10 * HeartbeatTick to avoid
    /// unnecessary leader switching
    pub election_tick: Option<usize>,

    /// HeartbeatTick is the number of node.tick invocations that must pass between
    /// heartbeats. That is, a leader sends heartbeat messages to maintain its
    /// leadership every heartbeat ticks.
    pub heartbeat_tick: Option<usize>,

    /// Limit the max size of each append message. Smaller value lowers
    /// the raft recovery cost(initial probing and message lost during normal operation).
    /// On the other side, it might affect the throughput during normal replication.
    /// Note: math.MaxUusize64 for unlimited, 0 for at most one entry per message.
    pub max_size_per_msg: Option<u64>,

    /// Limit the max number of in-flight append messages during optimistic
    /// replication phase. The application transportation layer usually has its own sending
    /// buffer over TCP/UDP. Set to avoid overflowing that sending buffer.
    /// TODO: feedback to application to limit the proposal rate?
    pub max_inflight_msgs: Option<usize>,

    /// Specify if the leader should check quorum activity. Leader steps down when
    /// quorum is not active for an electionTimeout.
    pub check_quorum: Option<bool>,

    /// Enables the Pre-Vote algorithm described in raft thesis section
    /// 9.6. This prevents disruption when a node that has been partitioned away
    /// rejoins the cluster.
    pub pre_vote: Option<bool>,

    /// The range of election timeout. In some cases, we hope some nodes has less possibility
    /// to become leader. This configuration ensures that the randomized election_timeout
    /// will always be suit in [min_election_tick, max_election_tick).
    /// If it is 0, then election_tick will be chosen.
    pub min_election_tick: Option<usize>,

    /// If it is 0, then 2 * election_tick will be chosen.
    pub max_election_tick: Option<usize>,

    /// Choose the linearizability mode or the lease mode to read data. If you don’t care about the read consistency and want a higher read performance, you can use the lease mode.
    ///
    /// Setting this to `LeaseBased` requires `check_quorum = true`.
    #[serde(
    default = "RaftConfig::read_only_option_default",
    serialize_with = "RaftConfig::serialize_read_only_option",
    deserialize_with = "RaftConfig::deserialize_read_only_option"
    )]
    pub read_only_option: ReadOnlyOption,

    /// Don't broadcast an empty raft entry to notify follower to commit an entry.
    /// This may make follower wait a longer time to apply an entry. This configuration
    /// May affect proposal forwarding and follower read.
    pub skip_bcast_commit: Option<bool>,

    /// Batches every append msg if any append msg already exists
    pub batch_append: Option<bool>,

    /// The election priority of this node.
    pub priority: Option<u64>,

    /// Specify maximum of uncommitted entry size.
    /// When this limit is reached, all proposals to append new log will be dropped
    pub max_uncommitted_size: Option<u64>,

    /// Max size for committed entries in a `Ready`.
    pub max_committed_size_per_ready: Option<u64>,
}

impl RaftConfig {
    pub(crate) fn to_raft_config(&self) -> rmqtt_raft::Config {
        let mut cfg = rmqtt_raft::Config { ..Default::default() };

        if let Some(grpc_timeout) = self.grpc_timeout {
            cfg.grpc_timeout = grpc_timeout;
        }
        if let Some(grpc_concurrency_limit) = self.grpc_concurrency_limit {
            cfg.grpc_concurrency_limit = grpc_concurrency_limit;
        }
        if let Some(grpc_breaker_threshold) = self.grpc_breaker_threshold {
            cfg.grpc_breaker_threshold = grpc_breaker_threshold;
        }
        if let Some(grpc_breaker_retry_interval) = self.grpc_breaker_retry_interval {
            cfg.grpc_breaker_retry_interval = grpc_breaker_retry_interval;
        }
        if let Some(proposal_batch_size) = self.proposal_batch_size {
            cfg.proposal_batch_size = proposal_batch_size;
        }
        if let Some(proposal_batch_timeout) = self.proposal_batch_timeout {
            cfg.proposal_batch_timeout = proposal_batch_timeout;
        }
        if let Some(snapshot_interval) = self.snapshot_interval {
            cfg.snapshot_interval = snapshot_interval;
        }
        if let Some(heartbeat) = self.heartbeat {
            cfg.heartbeat = heartbeat;
        }

        //---------------------------------------------------------------------------
        if let Some(election_tick) = self.election_tick {
            cfg.raft_cfg.election_tick = election_tick;
        }
        if let Some(heartbeat_tick) = self.heartbeat_tick {
            cfg.raft_cfg.heartbeat_tick = heartbeat_tick;
        }
        if let Some(max_size_per_msg) = self.max_size_per_msg {
            cfg.raft_cfg.max_size_per_msg = max_size_per_msg;
        }
        if let Some(max_inflight_msgs) = self.max_inflight_msgs {
            cfg.raft_cfg.max_inflight_msgs = max_inflight_msgs;
        }
        if let Some(check_quorum) = self.check_quorum {
            cfg.raft_cfg.check_quorum = check_quorum;
        }
        if let Some(pre_vote) = self.pre_vote {
            cfg.raft_cfg.pre_vote = pre_vote;
        }
        if let Some(min_election_tick) = self.min_election_tick {
            cfg.raft_cfg.min_election_tick = min_election_tick;
        }
        if let Some(max_election_tick) = self.max_election_tick {
            cfg.raft_cfg.max_election_tick = max_election_tick;
        }
        if let Some(skip_bcast_commit) = self.skip_bcast_commit {
            cfg.raft_cfg.skip_bcast_commit = skip_bcast_commit;
        }
        if let Some(batch_append) = self.batch_append {
            cfg.raft_cfg.batch_append = batch_append;
        }
        if let Some(priority) = self.priority {
            cfg.raft_cfg.priority = priority;
        }
        if let Some(max_uncommitted_size) = self.max_uncommitted_size {
            cfg.raft_cfg.max_uncommitted_size = max_uncommitted_size;
        }
        if let Some(max_committed_size_per_ready) = self.max_committed_size_per_ready {
            cfg.raft_cfg.max_committed_size_per_ready = max_committed_size_per_ready;
        }
        cfg.raft_cfg.read_only_option = self.read_only_option;
        cfg
    }

    fn read_only_option_default() -> ReadOnlyOption {
        ReadOnlyOption::Safe
    }

    pub fn deserialize_read_only_option<'de, D>(deserializer: D) -> Result<ReadOnlyOption, D::Error>
        where
            D: Deserializer<'de>,
    {
        let v = String::deserialize(deserializer)?.to_lowercase();
        match v.as_str() {
            "safe" => Ok(ReadOnlyOption::Safe),
            "leasebased" => Ok(ReadOnlyOption::LeaseBased),
            _ => Err(de::Error::missing_field("read_only_option")),
        }
    }

    #[inline]
    pub fn serialize_read_only_option<S>(rop: &ReadOnlyOption, s: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let rop_str = match rop {
            ReadOnlyOption::Safe => "safe",
            ReadOnlyOption::LeaseBased => "leasebased",
        };
        rop_str.serialize(s)
    }
}
