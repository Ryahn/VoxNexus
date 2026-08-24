//! In-memory gateway resume tokens + per-session event ring (F013 / F035).

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;
use voxnexus_protocol::Envelope;

const RESUME_TTL: Duration = Duration::from_secs(10 * 60);
/// Max fanout envelopes retained per resume session for replay.
const RING_CAPACITY: usize = 1_000;

#[derive(Debug, Clone)]
pub struct ResumeEntry {
    pub account_id: Uuid,
    pub gateway_session_id: Uuid,
    pub last_sequence: u64,
    expires_at: Instant,
    ring: VecDeque<Envelope>,
}

impl ResumeEntry {
    /// Events with `sequence > last_sequence` if the ring still covers the gap.
    ///
    /// Returns `None` when the client is too far behind (events dropped from the ring).
    #[must_use]
    pub fn replay_after(&self, last_sequence: u64) -> Option<Vec<Envelope>> {
        if self.ring.is_empty() {
            return Some(Vec::new());
        }
        let oldest = self.ring.front().map(|envelope| envelope.sequence)?;
        if last_sequence + 1 < oldest {
            return None;
        }
        Some(
            self.ring
                .iter()
                .filter(|envelope| envelope.sequence > last_sequence)
                .cloned()
                .collect(),
        )
    }
}

/// Process-local resume token registry with fanout event rings.
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
                ring: VecDeque::new(),
            },
        );
    }

    /// Append a fanout envelope to the ring for `token` (no-op if unknown).
    pub fn append(&self, token: &str, envelope: Envelope) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::purge_locked(&mut guard);
        let Some(entry) = guard.get_mut(token) else {
            return;
        };
        entry.last_sequence = entry.last_sequence.max(envelope.sequence);
        entry.ring.push_back(envelope);
        while entry.ring.len() > RING_CAPACITY {
            entry.ring.pop_front();
        }
        entry.expires_at = Instant::now() + RESUME_TTL;
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

#[cfg(test)]
mod tests {
    use super::*;
    use voxnexus_protocol::{EventType, HelloPayload, GATEWAY_PROTOCOL_VERSION};

    fn envelope(sequence: u64) -> Envelope {
        Envelope::new(
            sequence,
            EventType::Hello,
            HelloPayload {
                heartbeat_interval_ms: 15_000,
                protocol_version: GATEWAY_PROTOCOL_VERSION,
                session_id: Uuid::now_v7(),
            },
        )
    }

    #[test]
    fn replay_detects_gap_when_ring_advanced() {
        let entry = ResumeEntry {
            account_id: Uuid::now_v7(),
            gateway_session_id: Uuid::now_v7(),
            last_sequence: 5,
            expires_at: Instant::now() + RESUME_TTL,
            ring: (3..=5).map(envelope).collect(),
        };
        assert!(entry.replay_after(1).is_none());
        assert_eq!(entry.replay_after(2).expect("contiguous").len(), 3);
        assert!(entry.replay_after(5).expect("caught up").is_empty());
    }

    #[test]
    fn empty_ring_always_resumable() {
        let entry = ResumeEntry {
            account_id: Uuid::now_v7(),
            gateway_session_id: Uuid::now_v7(),
            last_sequence: 2,
            expires_at: Instant::now() + RESUME_TTL,
            ring: VecDeque::new(),
        };
        assert!(entry.replay_after(0).expect("empty").is_empty());
    }
}
