use std::convert::From as _f;
use std::net::SocketAddr;

use ntex_mqtt::v5;
use ntex_mqtt::v5::codec::{Auth, DisconnectReasonCode};
use rust_box::task_exec_queue::LocalSpawnExt;

use crate::{ClientInfo, MqttError, Result, Runtime, Session, SessionState};
use crate::broker::{inflight::MomentStatus, types::*};
use crate::broker::executor::get_handshake_exec_queue;
use crate::settings::listener::Listener;

#[inline]
async fn refused_ack<Io>(
    handshake: v5::Handshake<Io>,
    connect_info: &ConnectInfo,
    ack_code: ConnectAckReasonV5,
    reason: String,
) -> v5::HandshakeAck<Io, SessionState> {
    let new_ack_code = Runtime::instance()
        .extends
        .hook_mgr()
        .await
        .client_connack(connect_info, ConnectAckReason::V5(ack_code))
        .await;
    log::warn!(
        "{:?} Connection Refused, handshake, ack_code: {:?}, new_ack_code: {:?}, reason: {}",
        connect_info.id(),
        ack_code,
        new_ack_code,
        reason,
    );
    new_ack_code.v5_error_ack(handshake)
}

#[inline]
pub async fn handshake<Io: 'static>(
    listen_cfg: Listener,
    handshake: v5::Handshake<Io>,
    remote_addr: SocketAddr,
    local_addr: SocketAddr,
) -> Result<v5::HandshakeAck<Io, SessionState>, MqttError> {
    log::debug!(
        "new Connection: local_addr: {:?}, remote: {:?}, {:?}, listen_cfg: {:?}",
        local_addr,
        remote_addr,
        handshake,
        listen_cfg
    );

    let id = Id::new(
        Runtime::instance().node.id(),
        Some(local_addr),
        Some(remote_addr),
        handshake.packet().client_id.clone(),
        handshake.packet().username.clone(),
    );
    let id1 = id.clone();
    Runtime::instance().stats.handshakings.max_max(handshake.handshakings());

    let exec = get_handshake_exec_queue(local_addr.port(), listen_cfg.clone());

    let start = chrono::Local::now().timestamp_millis();
    let handshake_fut = async move {
        if (chrono::Local::now().timestamp_millis() - start) > listen_cfg.handshake_timeout() as i64 {
            Runtime::instance().metrics.client_handshaking_timeout_inc();
            return Err(MqttError::from("execute handshake timeout"));
        }
        _handshake(id, listen_cfg, handshake).await
    };

    match handshake_fut.spawn(&exec).result().await {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => {
            log::warn!("{:?} Connection Refused, handshake error, reason: {:?}", id1, e.to_string());
            Err(e)
        }
        Err(e) => {
            Runtime::instance().metrics.client_handshaking_timeout_inc();
            log::warn!("{:?} Connection Refused, handshake timeout, reason: {:?}", id1, e.to_string());
            Err(MqttError::from("Connection Refused, execute handshake timeout"))
        }
    }
}

#[inline]
pub async fn _handshake<Io: 'static>(
    id: Id,
    listen_cfg: Listener,
    mut handshake: v5::Handshake<Io>,
) -> Result<v5::HandshakeAck<Io, SessionState>, MqttError> {
    let connect_info = ConnectInfo::V5(id.clone(), Box::new(handshake.packet().clone()));

    //hook, client connect
    let _user_props = Runtime::instance().extends.hook_mgr().await.client_connect(&connect_info).await;

    if listen_cfg.max_clientid_len > 0 && id.client_id.len() > listen_cfg.max_clientid_len {
        return Ok(refused_ack(
            handshake,
            &connect_info,
            ConnectAckReasonV5::ClientIdentifierNotValid,
            "client_id is too long".into(),
        )
            .await);
    }

    //Extended Auth is not supported
    if handshake.packet().auth_method.is_some() {
        return Ok(refused_ack(
            handshake,
            &connect_info,
            ConnectAckReasonV5::BadAuthenticationMethod,
            "extended Auth is not supported".into(),
        )
            .await);
    }

    //hook, client authenticate
    let ack = Runtime::instance()
        .extends
        .hook_mgr()
        .await
        .client_authenticate(&connect_info, listen_cfg.allow_anonymous)
        .await;
    if !ack.success() {
        if let ConnectAckReason::V5(ack) = ack {
            return Ok(refused_ack(handshake, &connect_info, ack, "Authentication failed".into()).await);
        } else {
            unreachable!()
        }
    }

    let sink = handshake.sink();
    let packet = handshake.packet_mut();

    let mut entry = match { Runtime::instance().extends.shared().await.entry(id.clone()) }.try_lock().await {
        Err(e) => {
            return Ok(refused_ack(
                handshake,
                &connect_info,
                ConnectAckReasonV5::ServerUnavailable,
                format!("{:?}", e),
            )
                .await);
        }
        Ok(entry) => entry,
    };

    // Kick out the current session, if it exists
    let (session_present, offline_info) = match entry.kick(packet.clean_start, false).await {
        Err(e) => {
            return Ok(refused_ack(
                handshake,
                &connect_info,
                ConnectAckReasonV5::ServerUnavailable,
                format!("{:?}", e),
            )
                .await);
        }
        Ok(Some(offline_info)) => (!packet.clean_start, Some(offline_info)),
        Ok(None) => (false, None),
    };

    let connected_at = chrono::Local::now().timestamp_millis();
    let client = ClientInfo::new(connect_info, session_present, connected_at);

    let fitter =
        Runtime::instance().extends.fitter_mgr().await.get(client.clone(), id.clone(), listen_cfg.clone());

    log::debug!("{:?} offline_info: {:?}", id, offline_info);
    let created_at =
        if let Some(ref offline_info) = offline_info { offline_info.created_at } else { connected_at };

    let max_inflight = fitter.max_inflight();
    let session =
        Session::new(id, listen_cfg, fitter.max_mqueue_len(), max_inflight.get() as usize, created_at);

    let keep_alive = match fitter.keep_alive(&mut packet.keep_alive) {
        Ok(keep_alive) => keep_alive,
        Err(e) => {
            return Ok(refused_ack(
                handshake,
                &client.connect_info,
                ConnectAckReasonV5::ServerUnavailable,
                format!("{:?}", e),
            )
                .await);
        }
    };

    let hook = Runtime::instance().extends.hook_mgr().await.hook(&session, &client);

    if offline_info.is_none() {
        //hook, session created
        hook.session_created().await;
    }

    let (state, tx) =
        SessionState::new(session, client, Sink::V5(sink), hook, fitter).start(keep_alive).await;

    if let Err(e) = entry.set(state.session.clone(), tx, state.client.clone()).await {
        return Ok(refused_ack(
            handshake,
            &state.client.connect_info,
            ConnectAckReasonV5::ServerUnavailable,
            format!("{:?}", e),
        )
            .await);
    }

    if let Some(o) = offline_info {
        if let Err(e) = state.transfer_session_state(packet.clean_start, o).await {
            log::warn!("{:?} Failed to transfer session state, {:?}", state.id, e);
        }
    }

    //hook, client connack
    let _ = Runtime::instance()
        .extends
        .hook_mgr()
        .await
        .client_connack(&state.client.connect_info, ConnectAckReason::V5(ConnectAckReasonV5::Success))
        .await;

    //hook, client connected
    state.hook.client_connected().await;

    log::debug!("{:?} keep_alive: {}", state.id, keep_alive);
    let id = state.id.clone();
    let session_expiry_interval_secs = packet.session_expiry_interval_secs;
    let server_keepalive_sec = packet.keep_alive;
    let max_qos = state.listen_cfg.max_qos_allowed;
    let retain_available = Runtime::instance().extends.retain().await.is_supported(&state.listen_cfg);
    let max_packet_size = state.fitter.max_packet_size();
    let shared_subscription_available =
        Runtime::instance().extends.shared_subscription().await.is_supported(&state.listen_cfg);
    Ok(handshake.ack(state).keep_alive(keep_alive).with(|ack: &mut v5::codec::ConnectAck| {
        ack.session_present = session_present;
        ack.server_keepalive_sec = Some(server_keepalive_sec);
        ack.session_expiry_interval_secs = session_expiry_interval_secs;
        ack.receive_max = Some(max_inflight);
        ack.max_qos = Some(max_qos);
        ack.retain_available = Some(retain_available);
        ack.max_packet_size = Some(max_packet_size);
        //ack.assigned_client_id = None; //@TODO ... If the client ID is assigned by the broker, the server needs to return the client ID to the terminal.
        ack.topic_alias_max = 0; //@TODO ...
        ack.wildcard_subscription_available = Some(true);
        ack.subscription_identifiers_available = Some(false);
        ack.shared_subscription_available = Some(shared_subscription_available);

        log::debug!("{:?} handshake.ack: {:?}", id, ack);
    }))
}

async fn subscribes(
    state: &v5::Session<SessionState>,
    mut subs: v5::control::Subscribe,
) -> Result<v5::ControlResult> {
    let shared_subscription_supported =
        Runtime::instance().extends.shared_subscription().await.is_supported(&state.listen_cfg);
    for mut sub in subs.iter_mut() {
        let s = Subscribe::from_v5(sub.topic(), sub.options(), shared_subscription_supported)?;
        let sub_ret = state.subscribe(s).await?;
        if let Some(qos) = sub_ret.success() {
            sub.confirm(qos)
        } else {
            sub.fail(sub_ret.into_inner())
        }
    }
    Ok(subs.ack())
}

async fn unsubscribes(
    state: &v5::Session<SessionState>,
    unsubs: v5::control::Unsubscribe,
) -> Result<v5::ControlResult> {
    let shared_subscription_supported =
        Runtime::instance().extends.shared_subscription().await.is_supported(&state.listen_cfg);
    for topic_filter in unsubs.iter() {
        let unsub = Unsubscribe::from(topic_filter, shared_subscription_supported)?;
        state.unsubscribe(unsub).await?;
    }
    Ok(unsubs.ack())
}

pub async fn control_message<E: std::fmt::Debug>(
    state: v5::Session<SessionState>,
    ctrl_msg: v5::ControlMessage<E>,
) -> Result<v5::ControlResult, MqttError> {
    log::debug!("{:?} incoming control message -> {:?}", state.id, ctrl_msg);

    let _ = state.send(Message::Keepalive);

    let crs = match ctrl_msg {
        v5::ControlMessage::Auth(auth) => auth.ack(Auth::default()),
        v5::ControlMessage::Ping(ping) => ping.ack(),
        v5::ControlMessage::Subscribe(subs) => match subscribes(&state, subs).await {
            Err(e) => {
                state
                    .client
                    .add_disconnected_reason(Reason::from(format!("Subscribe failed, {:?}", e)))
                    .await;
                log::error!("{:?} Subscribe failed, reason: {:?}", state.id, e);
                return Err(e);
            }
            Ok(r) => r,
        },
        v5::ControlMessage::Unsubscribe(unsubs) => match unsubscribes(&state, unsubs).await {
            Err(e) => {
                state
                    .client
                    .add_disconnected_reason(Reason::from(format!("Unsubscribe failed, {:?}", e)))
                    .await;
                log::error!("{:?} Unsubscribe failed, reason: {:?}", state.id, e);
                return Err(e);
            }
            Ok(r) => r,
        },
        v5::ControlMessage::Disconnect(disconnect) => {
            //disconnect.packet().user_properties
            state.send(Message::Disconnect(Disconnect::V5(disconnect.packet().clone())))?;
            disconnect.ack()
        }
        v5::ControlMessage::Closed(closed) => {
            if let Err(e) = state.send(Message::Closed(Reason::from_static("Remote close connect"))) {
                log::debug!("{:?} Closed error, reason: {:?}", state.id, e);
            }
            closed.ack()
        }
        v5::ControlMessage::Error(err) => {
            if let Err(e) = state.send(Message::Closed(Reason::from(format!("{:?}", err.get_err())))) {
                log::debug!("{:?} Closed error, reason: {:?}", state.id, e);
            }
            err.ack(DisconnectReasonCode::ServerBusy)
        }
        v5::ControlMessage::ProtocolError(protocol_error) => {
            if let Err(e) =
            state.send(Message::Closed(Reason::from(format!("{:?}", protocol_error.get_ref()))))
            {
                log::debug!("{:?} Closed error, reason: {:?}", state.id, e);
            }
            protocol_error.ack()
        }
    };

    Ok(crs)
}

#[inline]
pub async fn publish(
    state: v5::Session<SessionState>,
    pub_msg: v5::PublishMessage,
) -> Result<v5::PublishResult, MqttError> {
    log::debug!("{:?} incoming publish message: {:?}", state.id, pub_msg);

    let _ = state.send(Message::Keepalive);

    match &pub_msg {
        v5::PublishMessage::Publish(publish) => {
            if let Err(e) = state.publish_v5(publish).await {
                log::error!(
                    "{:?} Publish failed, reason: {:?}",
                    state.id,
                    state.client.get_disconnected_reason().await
                );
                return Err(e);
            }
        }
        v5::PublishMessage::PublishAck(ack) => {
            if let Some(iflt_msg) = state.inflight_win.write().await.remove(&ack.packet_id.get()) {
                //hook, message_ack
                state.hook.message_acked(iflt_msg.from, &iflt_msg.publish).await;
            }
        }
        v5::PublishMessage::PublishReceived(ack) => {
            state.inflight_win.write().await.update_status(&ack.packet_id.get(), MomentStatus::UnComplete);
        }
        v5::PublishMessage::PublishComplete(ack2) => {
            if let Some(iflt_msg) = state.inflight_win.write().await.remove(&ack2.packet_id.get()) {
                //hook, message_ack
                state.hook.message_acked(iflt_msg.from, &iflt_msg.publish).await;
            }
        }
    }

    Ok(pub_msg.ack())
}
