//! Message mention parsing (F039).

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Mentions found in content (deduped).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MentionSet {
    pub account_ids: BTreeSet<Uuid>,
    pub role_ids: BTreeSet<Uuid>,
    pub everyone: bool,
    pub here: bool,
}

impl MentionSet {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.account_ids.is_empty() && self.role_ids.is_empty() && !self.everyone && !self.here
    }
}

/// Parse `@{uuid}`, `@&{uuid}`, `@everyone`, and `@here` tokens from content.
#[must_use]
pub fn parse_mentions(content: &str) -> MentionSet {
    let mut set = MentionSet::default();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'@' {
            i += 1;
            continue;
        }
        let rest = &content[i..];
        if rest.starts_with("@everyone") && is_token_end(rest, "@everyone".len()) {
            set.everyone = true;
            i += "@everyone".len();
            continue;
        }
        if rest.starts_with("@here") && is_token_end(rest, "@here".len()) {
            set.here = true;
            i += "@here".len();
            continue;
        }
        if let Some(id) = parse_brace_uuid(rest, "@&{") {
            set.role_ids.insert(id);
            i += "@&{".len() + 36 + 1;
            continue;
        }
        if let Some(id) = parse_brace_uuid(rest, "@{") {
            set.account_ids.insert(id);
            i += "@{".len() + 36 + 1;
            continue;
        }
        i += 1;
    }
    set
}

fn is_token_end(rest: &str, token_len: usize) -> bool {
    match rest.as_bytes().get(token_len) {
        None => true,
        Some(b) => !b.is_ascii_alphanumeric() && *b != b'_',
    }
}

fn parse_brace_uuid(rest: &str, prefix: &str) -> Option<Uuid> {
    let after = rest.strip_prefix(prefix)?;
    if after.len() < 37 || after.as_bytes().get(36) != Some(&b'}') {
        return None;
    }
    Uuid::parse_str(&after[..36]).ok()
}

/// API shape for mentions on a message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct MessageMentions {
    pub account_ids: Vec<Uuid>,
    pub role_ids: Vec<Uuid>,
    pub everyone: bool,
    pub here: bool,
}

impl From<MentionSet> for MessageMentions {
    fn from(set: MentionSet) -> Self {
        Self {
            account_ids: set.account_ids.into_iter().collect(),
            role_ids: set.role_ids.into_iter().collect(),
            everyone: set.everyone,
            here: set.here,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_role_and_broadcast() {
        let id = Uuid::nil();
        let role = Uuid::from_u128(2);
        let content = format!("hi @{{{id}}} and @&{{{role}}} @everyone @here");
        let set = parse_mentions(&content);
        assert!(set.account_ids.contains(&id));
        assert!(set.role_ids.contains(&role));
        assert!(set.everyone);
        assert!(set.here);
    }

    #[test]
    fn ignores_plain_at_name() {
        let set = parse_mentions("hello @nova how are you");
        assert!(set.is_empty());
    }
}
