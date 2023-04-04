#![deny(unsafe_code)]
#[macro_use]
extern crate serde;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Method, Url};
use serde::ser::Serialize;
use tokio::sync::RwLock;

use config::PluginConfig;
use rmqtt::ntex::util::ByteString;
use rmqtt::{ahash, async_trait, dashmap, lazy_static, log, reqwest, serde_json, tokio};
use rmqtt::{
    broker::hook::{Handler, HookResult, Parameter, Register, ReturnType, Type},
    broker::types::{
        AuthResult, ConnectInfo, Id, Password, PublishAclResult, SubscribeAckReason, SubscribeAclResult,
    },
    plugin::{DynPlugin, DynPluginResult, Plugin},
    MqttError, Result, Runtime, TopicName,
};

mod config;

type DashMap<K, V> = dashmap::DashMap<K, V, ahash::RandomState>;
type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
type HashSet<K> = std::collections::HashSet<K, ahash::RandomState>;

const IGNORE: &str = "ignore";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
enum ACLType {
    Sub = 1,
    Pub = 2,
}

impl ACLType {
    fn as_str(&self) -> &str {
        match self {
            Self::Sub => "1",
            Self::Pub => "2",
        }
    }
}

type CacheMap = Arc<DashMap<Id, CacheValue>>;

#[inline]
pub async fn register(
    runtime: &'static Runtime,
    name: &'static str,
    descr: &'static str,
    default_startup: bool,
    immutable: bool,
) -> Result<()> {
    runtime
        .plugins
        .register(name, default_startup, immutable, move || -> DynPluginResult {
            Box::pin(async move {
                AuthHttpPlugin::new(runtime, name, descr).await.map(|p| -> DynPlugin { Box::new(p) })
            })
        })
        .await?;
    Ok(())
}

struct AuthHttpPlugin {
    runtime: &'static Runtime,
    name: String,
    descr: String,
    register: Box<dyn Register>,
    cfg: Arc<RwLock<PluginConfig>>,
    cache_map: CacheMap,
}

impl AuthHttpPlugin {
    #[inline]
    async fn new<S: Into<String>>(runtime: &'static Runtime, name: S, descr: S) -> Result<Self> {
        let name = name.into();
        let cfg = Arc::new(RwLock::new(runtime.settings.plugins.load_config::<PluginConfig>(&name)?));
        log::debug!("{} AuthHttpPlugin cfg: {:?}", name, cfg.read().await);
        let register = runtime.extends.hook_mgr().await.register();
        let cache_map = Arc::new(DashMap::default());
        Ok(Self { runtime, name, descr: descr.into(), register, cfg, cache_map })
    }
}

#[async_trait]
impl Plugin for AuthHttpPlugin {
    #[inline]
    async fn init(&mut self) -> Result<()> {
        log::info!("{} init", self.name);
        let cfg = &self.cfg;
        let cache_map = &self.cache_map;

        let priority = cfg.read().await.priority;
        self.register
            .add_priority(Type::ClientAuthenticate, priority, Box::new(AuthHandler::new(cfg, cache_map)))
            .await;
        self.register
            .add_priority(Type::ClientSubscribeCheckAcl, priority, Box::new(AuthHandler::new(cfg, cache_map)))
            .await;
        self.register
            .add_priority(Type::MessagePublishCheckAcl, priority, Box::new(AuthHandler::new(cfg, cache_map)))
            .await;
        self.register
            .add_priority(Type::ClientDisconnected, priority, Box::new(AuthHandler::new(cfg, cache_map)))
            .await;

        Ok(())
    }

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    async fn get_config(&self) -> Result<serde_json::Value> {
        self.cfg.read().await.to_json()
    }

    #[inline]
    async fn load_config(&mut self) -> Result<()> {
        let new_cfg = self.runtime.settings.plugins.load_config::<PluginConfig>(&self.name)?;
        *self.cfg.write().await = new_cfg;
        log::debug!("load_config ok,  {:?}", self.cfg);
        Ok(())
    }

    #[inline]
    async fn start(&mut self) -> Result<()> {
        log::info!("{} start", self.name);
        self.register.start().await;
        Ok(())
    }

    #[inline]
    async fn stop(&mut self) -> Result<bool> {
        log::info!("{} stop", self.name);
        self.register.stop().await;
        Ok(true)
    }

    #[inline]
    fn version(&self) -> &str {
        "0.1.1"
    }

    #[inline]
    fn descr(&self) -> &str {
        &self.descr
    }

    #[inline]
    async fn attrs(&self) -> serde_json::Value {
        serde_json::json!({
            "cache_count": self.cache_map.len()
        })
    }
}

struct AuthHandler {
    cfg: Arc<RwLock<PluginConfig>>,
    cache_map: CacheMap,
}

impl AuthHandler {
    fn new(cfg: &Arc<RwLock<PluginConfig>>, cache_map: &CacheMap) -> Self {
        Self { cfg: cfg.clone(), cache_map: cache_map.clone() }
    }

    async fn http_get_request<T: Serialize + ?Sized>(
        url: Url,
        body: &T,
        headers: HeaderMap,
        timeout: Duration,
    ) -> Result<(bool, String)> {
        log::debug!("http_get_request, timeout: {:?}, url: {}", timeout, url);
        match HTTP_CLIENT.clone().get(url).headers(headers).timeout(timeout).query(body).send().await {
            Err(e) => {
                log::error!("error:{:?}", e);
                Err(MqttError::Msg(e.to_string()))
            }
            Ok(resp) => {
                let ok = resp.status().is_success();
                let body = resp.text().await.map_err(|e| MqttError::Msg(e.to_string()))?;
                Ok((ok, body))
            }
        }
    }

    async fn http_form_request<T: Serialize + ?Sized>(
        url: Url,
        method: Method,
        body: &T,
        headers: HeaderMap,
        timeout: Duration,
    ) -> Result<(bool, String)> {
        log::debug!("http_form_request, method: {:?}, timeout: {:?}, url: {}", method, timeout, url);
        match HTTP_CLIENT
            .clone()
            .request(method, url)
            .headers(headers)
            .timeout(timeout)
            .form(body)
            .send()
            .await
        {
            Err(e) => {
                log::error!("error:{:?}", e);
                Err(MqttError::Msg(e.to_string()))
            }
            Ok(resp) => {
                let ok = resp.status().is_success();
                let body = resp.text().await.map_err(|e| MqttError::Msg(e.to_string()))?;
                Ok((ok, body))
            }
        }
    }

    async fn http_json_request<T: Serialize + ?Sized>(
        url: Url,
        method: Method,
        body: &T,
        headers: HeaderMap,
        timeout: Duration,
    ) -> Result<(bool, String)> {
        log::debug!("http_json_request, method: {:?}, timeout: {:?}, url: {}", method, timeout, url);
        match HTTP_CLIENT
            .clone()
            .request(method, url)
            .headers(headers)
            .timeout(timeout)
            .json(body)
            .send()
            .await
        {
            Err(e) => {
                log::error!("error:{:?}", e);
                Err(MqttError::Msg(e.to_string()))
            }
            Ok(resp) => {
                let ok = resp.status().is_success();
                let body = resp.text().await.map_err(|e| MqttError::Msg(e.to_string()))?;
                Ok((ok, body))
            }
        }
    }

    fn replaces<'a>(
        params: &'a mut HashMap<String, String>,
        connect_info: &ConnectInfo,
        password: Option<&Password>,
        sub_or_pub: Option<(ACLType, &TopicName)>,
    ) -> Result<()> {
        let password =
            if let Some(p) = password { ByteString::try_from(p.clone())? } else { ByteString::default() };
        let client_id = connect_info.client_id();
        let username = connect_info.username().map(|n| n.as_ref()).unwrap_or("");
        let remote_addr = connect_info.id().remote_addr.map(|addr| addr.ip().to_string()).unwrap_or_default();
        for v in params.values_mut() {
            *v = v.replace("%u", username);
            *v = v.replace("%c", client_id);
            *v = v.replace("%a", &remote_addr);
            *v = v.replace("%r", "mqtt");
            *v = v.replace("%P", &password);
            if let Some((ref acl_type, topic)) = sub_or_pub {
                *v = v.replace("%A", acl_type.as_str());
                *v = v.replace("%t", topic);
            } else {
                *v = v.replace("%A", "");
                *v = v.replace("%t", "");
            }
        }
        Ok(())
    }

    async fn request(
        &self,
        connect_info: &ConnectInfo,
        mut req_cfg: config::Req,
        password: Option<&Password>,
        sub_or_pub: Option<(ACLType, &TopicName)>,
        check_super: bool,
    ) -> Result<bool> {
        log::debug!("{:?} req_cfg.url.path(): {:?}", connect_info.id(), req_cfg.url.path());
        let catch_key_fn =
            || CacheItem(req_cfg.url.path().to_owned(), sub_or_pub.map(|(f, t)| (f, t.clone())));
        let catch_key = {
            if let Some(cached) = self.cache_map.get(connect_info.id()) {
                log::debug!("{:?} catch value: {:?}", connect_info.id(), cached.value());
                if cached.is_super {
                    return Ok(true);
                }

                let catch_key = catch_key_fn();
                if cached.ignores.contains(&catch_key) {
                    return Ok(true);
                }
                catch_key
            } else {
                catch_key_fn()
            }
        };

        log::debug!("catch_key: {:?}", catch_key);

        let (headers, timeout) = {
            let cfg = self.cfg.read().await;
            let headers = match (cfg.headers(), req_cfg.headers()) {
                (Some(def_headers), Some(req_headers)) => {
                    let mut headers = def_headers.clone();
                    headers.extend(req_headers.clone());
                    headers
                }
                (Some(def_headers), None) => def_headers.clone(),
                (None, Some(req_headers)) => req_headers.clone(),
                (None, None) => HeaderMap::new(),
            };
            (headers, cfg.http_timeout)
        };

        let (allow, ignore) = if req_cfg.is_get() {
            let body = &mut req_cfg.params;
            Self::replaces(body, connect_info, password, sub_or_pub)?;
            Self::http_get_request(req_cfg.url, body, headers, timeout).await?
        } else if req_cfg.json_body() {
            let body = &mut req_cfg.params;
            Self::replaces(body, connect_info, password, sub_or_pub)?;
            Self::http_json_request(req_cfg.url, req_cfg.method, body, headers, timeout).await?
        } else {
            //form body
            let body = &mut req_cfg.params;
            Self::replaces(body, connect_info, password, sub_or_pub)?;
            Self::http_form_request(req_cfg.url, req_cfg.method, body, headers, timeout).await?
        };

        log::debug!("check_super: {}, allow: {:?}, ignore: {}", check_super, allow, ignore);

        //IGNORE
        if allow && ignore == IGNORE {
            let mut cached = self.cache_map.entry(connect_info.id().clone()).or_default();
            if check_super {
                cached.is_super = true;
            } else if sub_or_pub.is_some() {
                cached.ignores.insert(catch_key);
            }
        }

        Ok(allow)
    }

    async fn auth(&self, connect_info: &ConnectInfo, password: Option<&Password>) -> bool {
        if let Some(req) = { self.cfg.read().await.http_auth_req.clone() } {
            match self.request(connect_info, req.clone(), password, None, false).await {
                Ok(resp) => resp,
                Err(e) => {
                    log::warn!("{:?} auth error, {:?}", connect_info.id(), e);
                    false
                }
            }
        } else {
            true
        }
    }

    async fn is_super(&self, connect_info: &ConnectInfo) -> bool {
        if let Some(req) = { self.cfg.read().await.http_super_req.clone() } {
            match self.request(connect_info, req.clone(), None, None, true).await {
                Ok(resp) => resp,
                Err(e) => {
                    log::warn!("{:?} check super error, {:?}", connect_info.id(), e);
                    false
                }
            }
        } else {
            true
        }
    }

    async fn acl(&self, connect_info: &ConnectInfo, sub_or_pub: Option<(ACLType, &TopicName)>) -> bool {
        if let Some(req) = { self.cfg.read().await.http_acl_req.clone() } {
            match self.request(connect_info, req.clone(), None, sub_or_pub, false).await {
                Ok(allow) => {
                    log::debug!("acl.allow: {:?}", allow);
                    allow
                }
                Err(e) => {
                    log::warn!("{:?} acl error, {:?}", connect_info.id(), e);
                    false
                }
            }
        } else {
            true
        }
    }
}

#[async_trait]
impl Handler for AuthHandler {
    async fn hook(&self, param: &Parameter, acc: Option<HookResult>) -> ReturnType {
        match param {
            Parameter::ClientAuthenticate(connect_info) => {
                log::debug!("ClientAuthenticate auth-http");
                if matches!(
                    acc,
                    Some(HookResult::AuthResult(AuthResult::BadUsernameOrPassword))
                        | Some(HookResult::AuthResult(AuthResult::NotAuthorized))
                ) {
                    return (false, acc);
                }

                let stop = !self.cfg.read().await.break_if_allow;

                if self.is_super(*connect_info).await {
                    return (stop, Some(HookResult::AuthResult(AuthResult::Allow)));
                }

                return if !self.auth(*connect_info, connect_info.password()).await {
                    (false, Some(HookResult::AuthResult(AuthResult::BadUsernameOrPassword)))
                } else {
                    (stop, Some(HookResult::AuthResult(AuthResult::Allow)))
                };
            }

            Parameter::ClientSubscribeCheckAcl(_session, client_info, subscribe) => {
                if let Some(HookResult::SubscribeAclResult(acl_result)) = &acc {
                    if acl_result.failure() {
                        return (false, acc);
                    }
                }

                return if self
                    .acl(&client_info.connect_info, Some((ACLType::Sub, &subscribe.topic_filter)))
                    .await
                {
                    (
                        !self.cfg.read().await.break_if_allow,
                        Some(HookResult::SubscribeAclResult(SubscribeAclResult::new_success(subscribe.qos))),
                    )
                } else {
                    (
                        false,
                        Some(HookResult::SubscribeAclResult(SubscribeAclResult::new_failure(
                            SubscribeAckReason::NotAuthorized,
                        ))),
                    )
                };
            }

            Parameter::MessagePublishCheckAcl(_session, client_info, publish) => {
                log::debug!("MessagePublishCheckAcl");
                if let Some(HookResult::PublishAclResult(PublishAclResult::Rejected(_))) = &acc {
                    return (false, acc);
                }

                return if self.acl(&client_info.connect_info, Some((ACLType::Pub, publish.topic()))).await {
                    (
                        !self.cfg.read().await.break_if_allow,
                        Some(HookResult::PublishAclResult(PublishAclResult::Allow)),
                    )
                } else {
                    (
                        false,
                        Some(HookResult::PublishAclResult(PublishAclResult::Rejected(
                            self.cfg.read().await.disconnect_if_pub_rejected,
                        ))),
                    )
                };
            }
            Parameter::ClientDisconnected(_session, client_info, _reason) => {
                log::debug!("ClientDisconnected");
                self.cache_map.remove(&client_info.id);
            }
            _ => {
                log::error!("unimplemented, {:?}", param)
            }
        }
        (true, acc)
    }
}

lazy_static::lazy_static! {
    static ref  HTTP_CLIENT: reqwest::Client = {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap()
    };
}

type Path = String;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheItem(Path, Option<(ACLType, TopicName)>);

#[derive(Debug, Default)]
struct CacheValue {
    is_super: bool,
    ignores: HashSet<CacheItem>,
}
