//! Gateway WebSocket session: HELLO, heartbeat, and dev-only ping.

mod session;

pub use session::{missed_heartbeat, run_session, GatewaySessionOptions, HEARTBEAT_TIMEOUT_FACTOR};
