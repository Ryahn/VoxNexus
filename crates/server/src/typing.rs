//! Ephemeral typing indicators (F041).

use uuid::Uuid;
use voxnexus_auth::{ensure_profile, get_channel, list_member_account_ids};
use voxnexus_domain::ChannelType;
use voxnexus_permissions::PermissionCode;
use voxnexus_protocol::TypingStartPayload;
use voxnexus_realtime::PresenceHubMessage;

use crate::http::AppState;
use crate::permissions::allowed_for_channel;

pub async fn handle_typing_start(state: &AppState, account_id: Uuid, channel_id: Uuid) {
    let Ok(Some(channel)) = get_channel(&state.pool, channel_id).await else {
        return;
    };
    if channel.channel_type != ChannelType::Text || channel.archived_at.is_some() {
        return;
    }
    let Ok(true) =
        allowed_for_channel(state, &channel, account_id, PermissionCode::TEXT_SEND).await
    else {
        return;
    };
    if !state
        .presence_hub
        .allow_typing(account_id, channel_id)
        .await
    {
        return;
    }

    let display_name = match ensure_profile(&state.pool, account_id).await {
        Ok(profile) => {
            if profile.display_name.trim().is_empty() {
                "Someone".to_owned()
            } else {
                profile.display_name
            }
        }
        Err(_) => "Someone".to_owned(),
    };

    let members = match list_member_account_ids(&state.pool, channel.community_id).await {
        Ok(ids) => ids,
        Err(error) => {
            tracing::error!(error = %error, "list members for typing fanout failed");
            return;
        }
    };
    let mut recipients = Vec::new();
    for member_id in members {
        if member_id == account_id {
            continue;
        }
        if let Ok(true) =
            allowed_for_channel(state, &channel, member_id, PermissionCode::TEXT_VIEW).await
        {
            recipients.push(member_id);
        }
    }

    state
        .presence_hub
        .broadcast_to_accounts(
            &recipients,
            PresenceHubMessage::TypingStart(TypingStartPayload {
                channel_id,
                account_id,
                display_name,
            }),
        )
        .await;
}
