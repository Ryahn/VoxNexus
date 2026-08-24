//! In-memory permission snapshot cache (F029).

use std::collections::HashMap;
use std::sync::RwLock;

use uuid::Uuid;

use crate::eval::ActorContext;

#[derive(Debug, Clone)]
struct CachedActor {
    version: u64,
    context: ActorContext,
}

/// Per-community versioned cache of [`ActorContext`].
#[derive(Debug, Default)]
pub struct PermissionCache {
    community_versions: RwLock<HashMap<Uuid, u64>>,
    actors: RwLock<HashMap<(Uuid, Uuid), CachedActor>>,
}

impl PermissionCache {
    /// Bump cache generation for a community (role or membership changes).
    pub fn invalidate_community(&self, community_id: Uuid) {
        if let Ok(mut versions) = self.community_versions.write() {
            let next = versions.get(&community_id).copied().unwrap_or(0) + 1;
            versions.insert(community_id, next);
        }
    }

    /// Read cached context when `version` still matches.
    #[must_use]
    pub fn get(&self, community_id: Uuid, account_id: Uuid) -> Option<ActorContext> {
        let version = self
            .community_versions
            .read()
            .ok()
            .and_then(|map| map.get(&community_id).copied())
            .unwrap_or(0);
        let actors = self.actors.read().ok()?;
        let cached = actors.get(&(community_id, account_id))?;
        if cached.version == version {
            Some(cached.context.clone())
        } else {
            None
        }
    }

    /// Store a freshly loaded context.
    pub fn put(&self, context: ActorContext) {
        let version = self
            .community_versions
            .read()
            .ok()
            .and_then(|map| map.get(&context.community_id).copied())
            .unwrap_or(0);
        if let Ok(mut actors) = self.actors.write() {
            actors.insert(
                (context.community_id, context.account_id),
                CachedActor {
                    version,
                    context,
                },
            );
        }
    }
}
