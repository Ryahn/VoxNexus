//! Append-only audit event persistence (F033).

use chrono::Utc;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use voxnexus_domain::AuditEvent;

use crate::AuthError;
use sqlx::PgPool;

/// Input for a new audit row (immutable after insert).
#[derive(Debug, Clone)]
pub struct NewAuditEvent {
    pub community_id: Uuid,
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub space_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub summary: String,
    pub metadata: Value,
}

/// Filtered page of audit events (newest first).
#[derive(Debug, Clone)]
pub struct AuditEventsPage {
    pub items: Vec<AuditEvent>,
    pub has_more: bool,
}

/// Insert one audit event. Callers should treat failure as non-fatal for product writes.
///
/// # Errors
///
/// Returns database errors.
pub async fn insert_audit_event(
    pool: &PgPool,
    input: NewAuditEvent,
) -> Result<AuditEvent, AuthError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO audit_events (
            id, community_id, actor_account_id, action, space_id,
            target_type, target_id, summary, metadata, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ",
    )
    .bind(id)
    .bind(input.community_id)
    .bind(input.actor_account_id)
    .bind(&input.action)
    .bind(input.space_id)
    .bind(&input.target_type)
    .bind(input.target_id)
    .bind(&input.summary)
    .bind(&input.metadata)
    .bind(now)
    .execute(pool)
    .await?;
    get_audit_event(pool, id)
        .await?
        .ok_or(AuthError::Db(sqlx::Error::RowNotFound))
}

/// Load one audit event by id.
///
/// # Errors
///
/// Returns database errors.
pub async fn get_audit_event(pool: &PgPool, id: Uuid) -> Result<Option<AuditEvent>, AuthError> {
    let row = sqlx::query_as::<_, AuditEventRow>(
        r"
        SELECT id, community_id, actor_account_id, action, space_id,
               target_type, target_id, summary, metadata, created_at
        FROM audit_events
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AuditEventRow::into_event))
}

/// List audit events for a community (newest first), with optional filters.
///
/// # Errors
///
/// Returns database errors.
#[allow(clippy::too_many_arguments)]
pub async fn list_audit_events(
    pool: &PgPool,
    community_id: Uuid,
    after: Option<Uuid>,
    before: Option<Uuid>,
    limit: u16,
    actor_account_id: Option<Uuid>,
    action: Option<&str>,
    space_id: Option<Uuid>,
) -> Result<AuditEventsPage, AuthError> {
    let fetch = i64::from(limit) + 1;
    let action = action.map(str::trim).filter(|value| !value.is_empty());

    let rows = if let Some(after_id) = after {
        // after = older page (ids less than cursor when ordered DESC by id)
        sqlx::query_as::<_, AuditEventRow>(
            r"
            SELECT id, community_id, actor_account_id, action, space_id,
                   target_type, target_id, summary, metadata, created_at
            FROM audit_events
            WHERE community_id = $1
              AND id < $2
              AND ($3::uuid IS NULL OR actor_account_id = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::uuid IS NULL OR space_id = $5)
            ORDER BY id DESC
            LIMIT $6
            ",
        )
        .bind(community_id)
        .bind(after_id)
        .bind(actor_account_id)
        .bind(action)
        .bind(space_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    } else if let Some(before_id) = before {
        let mut rows = sqlx::query_as::<_, AuditEventRow>(
            r"
            SELECT id, community_id, actor_account_id, action, space_id,
                   target_type, target_id, summary, metadata, created_at
            FROM audit_events
            WHERE community_id = $1
              AND id > $2
              AND ($3::uuid IS NULL OR actor_account_id = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::uuid IS NULL OR space_id = $5)
            ORDER BY id ASC
            LIMIT $6
            ",
        )
        .bind(community_id)
        .bind(before_id)
        .bind(actor_account_id)
        .bind(action)
        .bind(space_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?;
        rows.reverse();
        rows
    } else {
        sqlx::query_as::<_, AuditEventRow>(
            r"
            SELECT id, community_id, actor_account_id, action, space_id,
                   target_type, target_id, summary, metadata, created_at
            FROM audit_events
            WHERE community_id = $1
              AND ($2::uuid IS NULL OR actor_account_id = $2)
              AND ($3::text IS NULL OR action = $3)
              AND ($4::uuid IS NULL OR space_id = $4)
            ORDER BY id DESC
            LIMIT $5
            ",
        )
        .bind(community_id)
        .bind(actor_account_id)
        .bind(action)
        .bind(space_id)
        .bind(fetch)
        .fetch_all(pool)
        .await?
    };

    let has_more = rows.len() > usize::from(limit);
    let items = rows
        .into_iter()
        .take(usize::from(limit))
        .map(AuditEventRow::into_event)
        .collect();
    Ok(AuditEventsPage { items, has_more })
}

#[derive(Debug, FromRow)]
struct AuditEventRow {
    id: Uuid,
    community_id: Uuid,
    actor_account_id: Option<Uuid>,
    action: String,
    space_id: Option<Uuid>,
    target_type: Option<String>,
    target_id: Option<Uuid>,
    summary: String,
    metadata: Value,
    created_at: chrono::DateTime<Utc>,
}

impl AuditEventRow {
    fn into_event(self) -> AuditEvent {
        AuditEvent {
            id: self.id,
            community_id: self.community_id,
            actor_account_id: self.actor_account_id,
            action: self.action,
            space_id: self.space_id,
            target_type: self.target_type,
            target_id: self.target_id,
            summary: self.summary,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}
