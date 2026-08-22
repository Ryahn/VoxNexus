//! Per-connection gateway loop.

use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, MissedTickBehavior};
use uuid::Uuid;
use voxnexus_protocol::{
    DevPingPayload, DevPongPayload, Envelope, EventType, HeartbeatAckPayload, HeartbeatPayload,
    HelloPayload, DEFAULT_HEARTBEAT_INTERVAL_MS, GATEWAY_PROTOCOL_VERSION,
};

/// Multiply [`DEFAULT_HEARTBEAT_INTERVAL_MS`] by this for the disconnect deadline.
pub const HEARTBEAT_TIMEOUT_FACTOR: u32 = 2;

/// Options for one gateway WebSocket session.
#[derive(Debug, Clone, Copy)]
pub struct GatewaySessionOptions {
    pub heartbeat_interval: Duration,
    pub allow_dev_ping: bool,
}

impl Default for GatewaySessionOptions {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            allow_dev_ping: false,
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
    // First tick completes immediately; skip so we wait a full interval before checking.
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
                            options.allow_dev_ping,
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

async fn handle_text(
    sink: &mut (impl SinkExt<Message> + Unpin),
    sequence: &mut u64,
    last_client_heartbeat: &mut Instant,
    allow_dev_ping: bool,
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
        EventType::DevPing if allow_dev_ping => {
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
        EventType::Hello | EventType::HeartbeatAck | EventType::DevPong | EventType::DevPing => {
            Err(false)
        }
    }
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
