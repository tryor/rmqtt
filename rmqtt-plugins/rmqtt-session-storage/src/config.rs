use rmqtt::serde_json;

use rmqtt_storage::Config;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    #[serde(default)]
    pub storage: Config,
}

impl PluginConfig {
    #[inline]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self)
    }
}
