//! Gateway WebSocket sessions, resume tokens, and (later) fanout.

mod presence;
mod resume;
mod session;

pub use presence::{PresenceHub, PresenceHubMessage, PresenceOutbound, TYPING_COOLDOWN};
pub use resume::{ResumeEntry, ResumeStore};
pub use session::{
    missed_heartbeat, run_session, GatewaySessionOptions, PresenceChangeHandler,
    TypingStartHandler, HEARTBEAT_TIMEOUT_FACTOR,
};
