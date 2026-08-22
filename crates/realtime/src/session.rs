//! Per-connection gateway loop.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use rand::RngCore;
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;
use voxnexus_protocol::{
    DevPingPayload, DevPongPayload, Envelope, EventType, HeartbeatAckPayload, HeartbeatPayload,
    HelloPayload, IdentifyPayload, InvalidSessionPayload, ReadyPayload, ResumePayload,
    ResumedPayload, DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION,
};

use crate::resume::ResumeStore;

/// Multiply heartbeat interval by this for the disconnect deadline.
pub const HEARTBEAT_TIMEOUT_FACTOR: u32 = 2;

/// Options for one gateway WebSocket session.
#[derive(Debug, Clone)]
pub struct GatewaySessionOptions {
    pub heartbeat_interval: Duration,
    /// Account bound from the HTTP session cookie on the upgrade.
    pub account_id: Uuid,
    /// When true, `DEV_PING` is accepted after identify (local protocol work).
    pub allow_dev_ping: bool,
    pub resume_store: Arc<ResumeStore>,
}

impl Default for GatewaySessionOptions {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            account_id: Uuid::nil(),
            allow_dev_ping: false,
            resume_store: Arc::new(ResumeStore::new()),
        }
    }
}

/// True when the client has gone silent longer than `interval * HEARTBEAT_TIMEOUT_FACTOR`.
#[must_use]
pub fn missed_heartbeat(last_client_heartbeat: Instant, interval: Duration, now: Instant) -> bool {
    now.duration_since(last_client_heartbeat) > interval * HEARTBEAT_TIMEOUT_FACTOR
}

/// Drive one gateway connection until close or heartbeat timeout.
pub async fn run_session(socket: WebSocket, options: GatewaySessionOptions) {
    let session_id = Uuid::now_v7();
    let mut sequence: u64 = 0;
    let mut last_client_heartbeat = Instant::now();
    let mut identified = false;
    let mut resume_token = String::new();

    let (mut sink, mut stream) = socket.split();

    sequence += 1;
    let hello = Envelope::new(
        sequence,
        EventType::Hello,
        HelloPayload {
            heartbeat_interval_ms: u64::try_from(options.heartbeat_interval.as_millis())
                .unwrap_or(u64::MAX),
            protocol_version: GATEWAY_PROTOCOL_VERSION,
            session_id,
        },
    );
    if send_envelope(&mut sink, &hello).await.is_err() {
        return;
    }

    let mut tick = interval(options.heartbeat_interval);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    tick.tick().await;

    loop {
        tokio::select! {
            maybe_msg = stream.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(fatal) = handle_text(
                            &mut sink,
                            &mut sequence,
                            &mut last_client_heartbeat,
                            &mut identified,
                            &mut resume_token,
                            session_id,
                            &options,
                            &text,
                        ).await {
                            if fatal {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if sink.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_) | Message::Binary(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(error)) => {
                        tracing::debug!(error = %error, %session_id, "gateway websocket error");
                        break;
                    }
                }
            }
            _ = tick.tick() => {
                if missed_heartbeat(last_client_heartbeat, options.heartbeat_interval, Instant::now()) {
                    tracing::debug!(%session_id, "gateway heartbeat timeout");
                    let _ = sink.send(Message::Close(None)).await;
                    break;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn handle_text(
    sink: &mut (impl SinkExt<Message> + Unpin),
    sequence: &mut u64,
    last_client_heartbeat: &mut Instant,
    identified: &mut bool,
    resume_token: &mut String,
    session_id: Uuid,
    options: &GatewaySessionOptions,
    text: &str,
) -> Result<(), bool> {
    let envelope: Envelope = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            tracing::debug!(error = %error, "gateway invalid envelope");
            return Err(false);
        }
    };

    match envelope.event_type {
        EventType::Heartbeat => {
            let _: HeartbeatPayload = serde_json::from_value(envelope.payload).unwrap_or_default();
            *last_client_heartbeat = Instant::now();
            *sequence += 1;
            let ack = Envelope::new(*sequence, EventType::HeartbeatAck, HeartbeatAckPayload {});
            send_envelope(sink, &ack).await.map_err(|()| true)?;
            Ok(())
        }
        EventType::Identify => {
            let _: IdentifyPayload = serde_json::from_value(envelope.payload).unwrap_or_default();
            if *identified {
                return Err(false);
            }
            *resume_token = new_resume_token();
            options.resume_store.put(
                resume_token.clone(),
                options.account_id,
                session_id,
                *sequence,
            );
            *identified = true;
            *sequence += 1;
            let ready = Envelope::new(
                *sequence,
                EventType::Ready,
                ReadyPayload {
                    account_id: options.account_id,
                    session_id,
                    resume_token: resume_token.clone(),
                },
            );
            send_envelope(sink, &ready).await.map_err(|()| true)?;
            Ok(())
        }
        EventType::Resume => {
            let request: ResumePayload = match serde_json::from_value(envelope.payload) {
                Ok(value) => value,
                Err(_) => return Err(false),
            };
            if *identified {
                return Err(false);
            }
            match options
                .resume_store
                .take_valid(&request.resume_token, options.account_id)
            {
                Some(entry) if entry.gateway_session_id == request.session_id => {
                    // F013: ring buffer may be empty; accept resume and reissue a token.
                    let _ = request.last_sequence;
                    *resume_token = new_resume_token();
                    options.resume_store.put(
                        resume_token.clone(),
                        options.account_id,
                        session_id,
                        *sequence,
                    );
                    *identified = true;
                    *sequence += 1;
                    let resumed =
                        Envelope::new(*sequence, EventType::Resumed, ResumedPayload { session_id });
                    send_envelope(sink, &resumed).await.map_err(|()| true)?;
                    Ok(())
                }
                _ => {
                    *sequence += 1;
                    let invalid = Envelope::new(
                        *sequence,
                        EventType::InvalidSession,
                        InvalidSessionPayload { resumable: false },
                    );
                    send_envelope(sink, &invalid).await.map_err(|()| true)?;
                    Err(true)
                }
            }
        }
        EventType::DevPing if options.allow_dev_ping && *identified => {
            let request: DevPingPayload = match serde_json::from_value(envelope.payload) {
                Ok(value) => value,
                Err(_) => return Err(false),
            };
            *sequence += 1;
            let reply = Envelope::new(
                *sequence,
                EventType::DevPong,
                DevPongPayload {
                    nonce: request.nonce,
                },
            );
            send_envelope(sink, &reply).await.map_err(|()| true)?;
            Ok(())
        }
        EventType::Hello
        | EventType::HeartbeatAck
        | EventType::Ready
        | EventType::Resumed
        | EventType::InvalidSession
        | EventType::DevPong
        | EventType::DevPing => Err(false),
    }
}

fn new_resume_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn send_envelope(
    sink: &mut (impl SinkExt<Message> + Unpin),
    envelope: &Envelope,
) -> Result<(), ()> {
    let text = serde_json::to_string(envelope).map_err(|_| ())?;
    sink.send(Message::Text(text.into())).await.map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn heartbeat_timeout_detects_silence() {
        let interval = Duration::from_secs(1);
        let last = Instant::now();
        assert!(!missed_heartbeat(
            last,
            interval,
            last + Duration::from_millis(500)
        ));
        assert!(!missed_heartbeat(
            last,
            interval,
            last + Duration::from_secs(2)
        ));
        assert!(missed_heartbeat(
            last,
            interval,
            last + Duration::from_millis(2001)
        ));
    }
}
