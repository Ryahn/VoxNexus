//! Gateway WebSocket sessions, resume tokens, and (later) fanout.

mod presence;
mod resume;
mod session;

pub use presence::{PresenceHub, PresenceHubMessage, PresenceOutbound};
pub use resume::{ResumeEntry, ResumeStore};
pub use session::{missed_heartbeat, run_session, GatewaySessionOptions, HEARTBEAT_TIMEOUT_FACTOR};
