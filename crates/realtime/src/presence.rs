//! In-memory instance presence (F018).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;
use voxnexus_domain::{PresenceStatus, PublicPresenceStatus};
use voxnexus_protocol::{MemberJoinPayload, MemberLeavePayload, PresenceUpdatePayload};

/// Outbound gateway fanout for one WebSocket connection.
pub type PresenceOutbound = mpsc::UnboundedSender<PresenceHubMessage>;

/// Messages pushed to a single gateway session.
#[derive(Debug, Clone)]
pub enum PresenceHubMessage {
    /// Initial snapshot after identify.
    Sync(Vec<PresenceUpdatePayload>),
    /// Incremental presence change.
    Update(PresenceUpdatePayload),
    /// Account joined a community (F020).
    MemberJoin(MemberJoinPayload),
    /// Account left a community (F020).
    MemberLeave(MemberLeavePayload),
}

#[derive(Clone)]
pub struct PresenceHub {
    inner: Arc<RwLock<HashMap<Uuid, AccountPresence>>>,
    grace: Duration,
}

struct AccountPresence {
    stored_status: PresenceStatus,
    custom_status: String,
    connections: HashMap<Uuid, PresenceOutbound>,
    grace_task: Option<JoinHandle<()>>,
}

impl PresenceHub {
    /// Create a hub that marks accounts offline after `grace` with no connections.
    #[must_use]
    pub fn new(grace: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            grace,
        }
    }

    /// Default grace matching gateway heartbeat timeout (2× default interval).
    #[must_use]
    pub fn with_default_grace() -> Self {
        Self::new(
            Duration::from_millis(voxnexus_protocol::DEFAULT_HEARTBEAT_INTERVAL_MS)
                * crate::session::HEARTBEAT_TIMEOUT_FACTOR,
        )
    }

    fn payload(
        account_id: Uuid,
        stored: PresenceStatus,
        custom_status: &str,
        self_view: bool,
        connected: bool,
    ) -> PresenceUpdatePayload {
        let status = if connected {
            PublicPresenceStatus::from_connected(stored, self_view)
        } else {
            PublicPresenceStatus::offline()
        };
        PresenceUpdatePayload {
            account_id,
            status,
            custom_status: custom_status.to_owned(),
        }
    }

    fn is_listed_online(stored: PresenceStatus, connected: bool) -> bool {
        connected && stored != PresenceStatus::Invisible
    }

    async fn broadcast_update(&self, payload: &PresenceUpdatePayload, except_conn: Option<Uuid>) {
        let inner = self.inner.read().await;
        for account in inner.values() {
            for (conn_id, outbound) in &account.connections {
                if except_conn == Some(*conn_id) {
                    continue;
                }
                let _ = outbound.send(PresenceHubMessage::Update(payload.clone()));
            }
        }
    }

    /// Fan out a hub message to every live connection for the given accounts.
    pub async fn broadcast_to_accounts(&self, account_ids: &[Uuid], message: PresenceHubMessage) {
        if account_ids.is_empty() {
            return;
        }
        let id_set: std::collections::HashSet<Uuid> = account_ids.iter().copied().collect();
        let inner = self.inner.read().await;
        for (account_id, account) in inner.iter() {
            if !id_set.contains(account_id) {
                continue;
            }
            for outbound in account.connections.values() {
                let _ = outbound.send(message.clone());
            }
        }
    }

    /// Register a live gateway connection and return the initial sync for this client.
    pub async fn connect(
        &self,
        account_id: Uuid,
        conn_id: Uuid,
        outbound: PresenceOutbound,
        stored_status: PresenceStatus,
        custom_status: String,
    ) -> Vec<PresenceUpdatePayload> {
        let (sync, announce) = {
            let mut inner = self.inner.write().await;
            let entry = inner.entry(account_id).or_insert_with(|| AccountPresence {
                stored_status,
                custom_status: custom_status.clone(),
                connections: HashMap::new(),
                grace_task: None,
            });

            if let Some(task) = entry.grace_task.take() {
                task.abort();
            }

            entry.stored_status = stored_status;
            entry.custom_status.clone_from(&custom_status);
            entry.connections.insert(conn_id, outbound);

            let sync: Vec<PresenceUpdatePayload> = inner
                .iter()
                .filter(|(_, state)| !state.connections.is_empty())
                .map(|(id, state)| {
                    let self_view = *id == account_id;
                    Self::payload(
                        *id,
                        state.stored_status,
                        &state.custom_status,
                        self_view,
                        true,
                    )
                })
                .collect();

            let announce = if Self::is_listed_online(stored_status, true) {
                Some(Self::payload(
                    account_id,
                    stored_status,
                    &custom_status,
                    false,
                    true,
                ))
            } else {
                None
            };
            (sync, announce)
        };

        if let Some(payload) = announce {
            self.broadcast_update(&payload, Some(conn_id)).await;
        }

        sync
    }

    /// Remove a gateway connection; schedule offline when the last tab disconnects.
    pub async fn disconnect(&self, account_id: Uuid, conn_id: Uuid) {
        let mut inner = self.inner.write().await;
        let Some(entry) = inner.get_mut(&account_id) else {
            return;
        };
        entry.connections.remove(&conn_id);
        if !entry.connections.is_empty() {
            return;
        }

        let stored = entry.stored_status;
        let custom_status = entry.custom_status.clone();
        let grace = self.grace;
        let hub = self.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(grace).await;
            hub.finalize_offline(account_id, stored, custom_status)
                .await;
        });
        entry.grace_task = Some(task);
    }

    async fn finalize_offline(
        &self,
        account_id: Uuid,
        stored: PresenceStatus,
        custom_status: String,
    ) {
        let should_broadcast = {
            let mut inner = self.inner.write().await;
            let remove = inner
                .get(&account_id)
                .is_some_and(|entry| entry.connections.is_empty());
            if remove {
                inner.remove(&account_id);
            }
            remove
        };
        if should_broadcast {
            let payload = Self::payload(account_id, stored, &custom_status, false, false);
            self.broadcast_update(&payload, None).await;
        }
    }

    /// Update stored presence for a connected account and fan out.
    pub async fn update(
        &self,
        account_id: Uuid,
        status: Option<PresenceStatus>,
        custom_status: Option<&str>,
    ) -> Option<PresenceUpdatePayload> {
        let (self_payload, public_payload) = {
            let mut inner = self.inner.write().await;
            let entry = inner.get_mut(&account_id)?;
            if entry.connections.is_empty() {
                return None;
            }
            if let Some(status) = status {
                entry.stored_status = status;
            }
            if let Some(custom) = custom_status {
                custom.clone_into(&mut entry.custom_status);
            }
            let stored = entry.stored_status;
            let custom = entry.custom_status.clone();
            let self_payload = Self::payload(account_id, stored, &custom, true, true);
            for outbound in entry.connections.values() {
                let _ = outbound.send(PresenceHubMessage::Update(self_payload.clone()));
            }
            let public_payload = if Self::is_listed_online(stored, true) {
                Some(Self::payload(account_id, stored, &custom, false, true))
            } else {
                None
            };
            (self_payload, public_payload)
        };

        if let Some(payload) = public_payload {
            self.broadcast_update(&payload, None).await;
        }
        Some(self_payload)
    }

    /// Public online list for HTTP (invisible accounts omitted).
    pub async fn list_public(&self) -> Vec<PresenceUpdatePayload> {
        let inner = self.inner.read().await;
        inner
            .iter()
            .filter(|(_, state)| {
                Self::is_listed_online(state.stored_status, !state.connections.is_empty())
            })
            .map(|(id, state)| {
                Self::payload(*id, state.stored_status, &state.custom_status, false, true)
            })
            .collect()
    }

    /// Whether `account_id` is connected and not invisible.
    pub async fn is_visible_online(&self, account_id: Uuid) -> bool {
        let inner = self.inner.read().await;
        inner.get(&account_id).is_some_and(|state| {
            Self::is_listed_online(state.stored_status, !state.connections.is_empty())
        })
    }

    /// Resolve presence for HTTP (offline when not connected).
    pub async fn view(
        &self,
        account_id: Uuid,
        viewer_id: Uuid,
        custom_fallback: &str,
    ) -> (PublicPresenceStatus, String) {
        let inner = self.inner.read().await;
        if let Some(state) = inner.get(&account_id) {
            if !state.connections.is_empty() {
                let payload = Self::payload(
                    account_id,
                    state.stored_status,
                    &state.custom_status,
                    account_id == viewer_id,
                    true,
                );
                return (payload.status, payload.custom_status);
            }
        }
        (PublicPresenceStatus::offline(), custom_fallback.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;
    use uuid::Uuid;
    use voxnexus_domain::PresenceStatus;

    use super::*;

    #[tokio::test]
    async fn disconnect_drops_from_public_list() {
        let hub = PresenceHub::new(Duration::from_millis(40));
        let (tx, _rx) = mpsc::unbounded_channel();
        let account_id = Uuid::now_v7();
        let conn_id = Uuid::now_v7();

        hub.connect(
            account_id,
            conn_id,
            tx,
            PresenceStatus::Online,
            String::new(),
        )
        .await;
        assert!(hub.is_visible_online(account_id).await);

        hub.disconnect(account_id, conn_id).await;
        assert!(!hub.is_visible_online(account_id).await);
        assert!(!hub
            .list_public()
            .await
            .iter()
            .any(|entry| entry.account_id == account_id));
    }

    #[tokio::test]
    async fn reconnect_during_grace_keeps_online() {
        let hub = PresenceHub::new(Duration::from_millis(80));
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let account_id = Uuid::now_v7();
        let conn1 = Uuid::now_v7();
        let conn2 = Uuid::now_v7();

        hub.connect(
            account_id,
            conn1,
            tx1,
            PresenceStatus::Online,
            String::new(),
        )
        .await;
        hub.disconnect(account_id, conn1).await;

        let (tx2, _rx2) = mpsc::unbounded_channel();
        hub.connect(
            account_id,
            conn2,
            tx2,
            PresenceStatus::Online,
            String::new(),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(120)).await;
        assert!(hub.is_visible_online(account_id).await);
    }

    #[tokio::test]
    async fn update_while_online_does_not_deadlock() {
        let hub = PresenceHub::new(Duration::from_millis(40));
        let (tx, mut rx) = mpsc::unbounded_channel();
        let account_id = Uuid::now_v7();
        let conn_id = Uuid::now_v7();

        hub.connect(
            account_id,
            conn_id,
            tx,
            PresenceStatus::Online,
            String::new(),
        )
        .await;

        let updated = tokio::time::timeout(
            Duration::from_secs(1),
            hub.update(account_id, Some(PresenceStatus::Idle), Some("afk")),
        )
        .await
        .expect("presence update deadlocked on hub RwLock")
        .expect("connected account");

        assert_eq!(updated.status, PublicPresenceStatus::Idle);
        assert_eq!(updated.custom_status, "afk");

        // Drain sync + self update (+ public broadcast to same conn).
        while rx.try_recv().is_ok() {}
    }
}
