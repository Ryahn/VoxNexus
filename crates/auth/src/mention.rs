//! Persist and load message mentions (F039).

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use sqlx::PgPool;
use uuid::Uuid;
use voxnexus_protocol::{MentionSet, MessageMentions};

use crate::AuthError;

/// Replace mention rows for a message.
pub async fn replace_message_mentions(
    pool: &PgPool,
    message_id: Uuid,
    mentions: &MentionSet,
) -> Result<(), AuthError> {
    sqlx::query("DELETE FROM message_mentions WHERE message_id = $1")
        .bind(message_id)
        .execute(pool)
        .await?;

    for account_id in &mentions.account_ids {
        sqlx::query(
            r"
            INSERT INTO message_mentions (message_id, kind, target_id)
            VALUES ($1, 'user', $2)
            ",
        )
        .bind(message_id)
        .bind(account_id)
        .execute(pool)
        .await?;
    }
    for role_id in &mentions.role_ids {
        sqlx::query(
            r"
            INSERT INTO message_mentions (message_id, kind, target_id)
            VALUES ($1, 'role', $2)
            ",
        )
        .bind(message_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    }
    if mentions.everyone {
        sqlx::query(
            r"
            INSERT INTO message_mentions (message_id, kind, target_id)
            VALUES ($1, 'everyone', NULL)
            ",
        )
        .bind(message_id)
        .execute(pool)
        .await?;
    }
    if mentions.here {
        sqlx::query(
            r"
            INSERT INTO message_mentions (message_id, kind, target_id)
            VALUES ($1, 'here', NULL)
            ",
        )
        .bind(message_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Load mentions for many messages.
pub async fn list_mentions_for_messages(
    pool: &PgPool,
    message_ids: &[Uuid],
) -> Result<Vec<(Uuid, MessageMentions)>, AuthError> {
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, MentionRow>(
        r"
        SELECT message_id, kind, target_id
        FROM message_mentions
        WHERE message_id = ANY($1)
        ",
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await?;

    let mut map: std::collections::BTreeMap<Uuid, MessageMentions> =
        std::collections::BTreeMap::new();
    for id in message_ids {
        map.entry(*id).or_default();
    }
    for row in rows {
        let entry = map.entry(row.message_id).or_default();
        match row.kind.as_str() {
            "user" => {
                if let Some(id) = row.target_id {
                    entry.account_ids.push(id);
                }
            }
            "role" => {
                if let Some(id) = row.target_id {
                    entry.role_ids.push(id);
                }
            }
            "everyone" => entry.everyone = true,
            "here" => entry.here = true,
            _ => {}
        }
    }
    Ok(map.into_iter().collect())
}

#[derive(Debug, sqlx::FromRow)]
struct MentionRow {
    message_id: Uuid,
    kind: String,
    target_id: Option<Uuid>,
}
