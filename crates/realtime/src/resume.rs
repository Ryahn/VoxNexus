//! In-memory gateway resume tokens (F013). Event replay lands in F035.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;

const RESUME_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone)]
pub struct ResumeEntry {
    pub account_id: Uuid,
    pub gateway_session_id: Uuid,
    pub last_sequence: u64,
    expires_at: Instant,
}

/// Process-local resume token registry.
#[derive(Debug, Default)]
pub struct ResumeStore {
    inner: Mutex<HashMap<String, ResumeEntry>>,
}

impl ResumeStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a resume token for an identified gateway session.
    pub fn put(
        &self,
        token: String,
        account_id: Uuid,
        gateway_session_id: Uuid,
        last_sequence: u64,
    ) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::purge_locked(&mut guard);
        guard.insert(
            token,
            ResumeEntry {
                account_id,
                gateway_session_id,
                last_sequence,
                expires_at: Instant::now() + RESUME_TTL,
            },
        );
    }

    /// Take a live entry for `token` if it matches `account_id` and has not expired.
    pub fn take_valid(&self, token: &str, account_id: Uuid) -> Option<ResumeEntry> {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::purge_locked(&mut guard);
        let entry = guard.remove(token)?;
        if entry.account_id != account_id || Instant::now() > entry.expires_at {
            return None;
        }
        Some(entry)
    }

    fn purge_locked(map: &mut HashMap<String, ResumeEntry>) {
        let now = Instant::now();
        map.retain(|_, entry| entry.expires_at > now);
    }
}
