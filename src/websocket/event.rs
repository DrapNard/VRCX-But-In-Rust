use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineError {
    pub err: String,

    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponseContent {
    pub notification_id: String,
    pub receiver_id: String,
    pub response_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationV2Update {
    pub id: String,
    pub version: u32,
    pub updates: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationV2Delete {
    pub ids: Vec<String>,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserContent {
    pub user_id: String,
    pub user: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendOnlineContent {
    pub user_id: String,
    pub platform: String,
    pub location: String,
    pub can_request_invite: bool,
    pub user: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendActiveContent {
    #[serde(alias = "userid")]
    pub user_id: String,
    pub platform: String,
    pub user: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendOfflineContent {
    pub user_id: String,
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendLocationContent {
    pub user_id: String,
    pub location: String,
    pub traveling_to_location: String,
    pub world_id: String,
    pub can_request_invite: bool,
    pub user: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLocationContent {
    pub user_id: String,
    pub user: Value,
    pub location: String,
    pub instance: String,
    pub traveling_to_location: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeAssignedContent {
    pub badge: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeUnassignedContent {
    pub badge_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentRefreshContent {
    pub content_type: String,
    pub file_id: Option<String>,
    pub item_id: Option<String>,
    pub item_type: Option<String>,
    pub action_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomyUpdateContent {
    pub dirty_purchases: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedImageUpdateContent {
    pub file_id: String,
    pub pixel_size: u64,
    pub version_number: u64,
    pub needs_processing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceQueueJoinedContent {
    pub instance_location: String,
    pub position: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceQueueReadyContent {
    pub instance_location: String,
    pub expiry: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupIdContent {
    pub group_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMemberUpdatedContent {
    pub member: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRoleUpdatedContent {
    pub role: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelineEvent {
    Notification(Value),
    ResponseNotification(NotificationResponseContent),
    SeeNotification(String),
    HideNotification(String),
    ClearNotification,
    NotificationV2(Value),
    NotificationV2Update(NotificationV2Update),
    NotificationV2Delete(NotificationV2Delete),
    FriendAdd(UserContent),
    FriendDelete { user_id: String },
    FriendOnline(FriendOnlineContent),
    FriendActive(FriendActiveContent),
    FriendOffline(FriendOfflineContent),
    FriendUpdate(UserContent),
    FriendLocation(FriendLocationContent),
    UserUpdate(UserContent),
    UserLocation(UserLocationContent),
    UserBadgeAssigned(BadgeAssignedContent),
    UserBadgeUnassigned(BadgeUnassignedContent),
    ContentRefresh(ContentRefreshContent),
    EconomyUpdate(EconomyUpdateContent),
    ModifiedImageUpdate(ModifiedImageUpdateContent),
    InstanceQueueJoined(InstanceQueueJoinedContent),
    InstanceQueueReady(InstanceQueueReadyContent),
    GroupJoined(GroupIdContent),
    GroupLeft(GroupIdContent),
    GroupMemberUpdated(GroupMemberUpdatedContent),
    GroupRoleUpdated(GroupRoleUpdatedContent),
    Unknown { event_type: String, content: Value },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelineMessage {
    Event(PipelineEvent),
    Error(PipelineError),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PipelineParseError {
    #[error("message is not a JSON object")]
    NotAnObject,

    #[error("missing pipeline event type")]
    MissingType,

    #[error("invalid JSON: {0}")]
    Json(String),

    #[error("invalid content for {event_type}: {reason}")]
    InvalidContent {
        event_type: String,
        reason: String,
        content: Value,
    },
}

pub fn parse_pipeline_message(message: &str) -> Result<PipelineMessage, PipelineParseError> {
    let raw = serde_json::from_str::<Value>(message)
        .map_err(|err| PipelineParseError::Json(err.to_string()))?;

    let object = raw.as_object().ok_or(PipelineParseError::NotAnObject)?;

    if object.contains_key("err") {
        let error = serde_json::from_value::<PipelineError>(raw)
            .map_err(|err| PipelineParseError::Json(err.to_string()))?;
        return Ok(PipelineMessage::Error(error));
    }

    let event_type = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or(PipelineParseError::MissingType)?;

    let content = normalize_content(object.get("content").cloned().unwrap_or(Value::Null))
        .map_err(|err| PipelineParseError::InvalidContent {
            event_type: event_type.to_string(),
            reason: err,
            content: object.get("content").cloned().unwrap_or(Value::Null),
        })?;

    let event = parse_event(event_type, content)?;

    Ok(PipelineMessage::Event(event))
}

fn normalize_content(content: Value) -> Result<Value, String> {
    match content {
        Value::String(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => Ok(value),
            Err(_) => Ok(Value::String(text)),
        },
        other => Ok(other),
    }
}

fn parse_event(event_type: &str, content: Value) -> Result<PipelineEvent, PipelineParseError> {
    let event = match event_type {
        "notification" => PipelineEvent::Notification(content),
        "response-notification" => {
            PipelineEvent::ResponseNotification(parse_content(event_type, content)?)
        }
        "see-notification" => {
            PipelineEvent::SeeNotification(parse_string_content(event_type, content)?)
        }
        "hide-notification" => {
            PipelineEvent::HideNotification(parse_string_content(event_type, content)?)
        }
        "clear-notification" => PipelineEvent::ClearNotification,
        "notification-v2" => PipelineEvent::NotificationV2(content),
        "notification-v2-update" => {
            PipelineEvent::NotificationV2Update(parse_content(event_type, content)?)
        }
        "notification-v2-delete" => {
            PipelineEvent::NotificationV2Delete(parse_content(event_type, content)?)
        }
        "friend-add" => PipelineEvent::FriendAdd(parse_content(event_type, content)?),
        "friend-delete" => {
            let content: UserIdOnly = parse_content(event_type, content)?;
            PipelineEvent::FriendDelete {
                user_id: content.user_id,
            }
        }
        "friend-online" => PipelineEvent::FriendOnline(parse_content(event_type, content)?),
        "friend-active" => PipelineEvent::FriendActive(parse_content(event_type, content)?),
        "friend-offline" => PipelineEvent::FriendOffline(parse_content(event_type, content)?),
        "friend-update" => PipelineEvent::FriendUpdate(parse_content(event_type, content)?),
        "friend-location" => PipelineEvent::FriendLocation(parse_content(event_type, content)?),
        "user-update" => PipelineEvent::UserUpdate(parse_content(event_type, content)?),
        "user-location" => PipelineEvent::UserLocation(parse_content(event_type, content)?),
        "user-badge-assigned" => {
            PipelineEvent::UserBadgeAssigned(parse_content(event_type, content)?)
        }
        "user-badge-unassigned" => {
            PipelineEvent::UserBadgeUnassigned(parse_content(event_type, content)?)
        }
        "content-refresh" => PipelineEvent::ContentRefresh(parse_content(event_type, content)?),
        "economy-update" => PipelineEvent::EconomyUpdate(parse_content(event_type, content)?),
        "modified-image-update" => {
            PipelineEvent::ModifiedImageUpdate(parse_content(event_type, content)?)
        }
        "instance-queue-joined" => {
            PipelineEvent::InstanceQueueJoined(parse_content(event_type, content)?)
        }
        "instance-queue-ready" => {
            PipelineEvent::InstanceQueueReady(parse_content(event_type, content)?)
        }
        "group-joined" => PipelineEvent::GroupJoined(parse_content(event_type, content)?),
        "group-left" => PipelineEvent::GroupLeft(parse_content(event_type, content)?),
        "group-member-updated" => {
            PipelineEvent::GroupMemberUpdated(parse_content(event_type, content)?)
        }
        "group-role-updated" => {
            PipelineEvent::GroupRoleUpdated(parse_content(event_type, content)?)
        }
        other => PipelineEvent::Unknown {
            event_type: other.to_string(),
            content,
        },
    };

    Ok(event)
}

fn parse_content<T>(event_type: &str, content: Value) -> Result<T, PipelineParseError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(content.clone()).map_err(|err| PipelineParseError::InvalidContent {
        event_type: event_type.to_string(),
        reason: err.to_string(),
        content,
    })
}

fn parse_string_content(event_type: &str, content: Value) -> Result<String, PipelineParseError> {
    match content {
        Value::String(value) => Ok(value),
        other => Err(PipelineParseError::InvalidContent {
            event_type: event_type.to_string(),
            reason: "expected string content".to_string(),
            content: other,
        }),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserIdOnly {
    user_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_double_encoded_content() {
        let message = r#"{"type":"friend-delete","content":"{\"userId\":\"usr_123\"}"}"#;

        let parsed = parse_pipeline_message(message).unwrap();

        assert_eq!(
            parsed,
            PipelineMessage::Event(PipelineEvent::FriendDelete {
                user_id: "usr_123".to_string()
            })
        );
    }

    #[test]
    fn keeps_plain_string_notification_ids() {
        let message = r#"{"type":"see-notification","content":"not_123"}"#;

        let parsed = parse_pipeline_message(message).unwrap();

        assert_eq!(
            parsed,
            PipelineMessage::Event(PipelineEvent::SeeNotification("not_123".to_string()))
        );
    }

    #[test]
    fn parses_pipeline_errors() {
        let message = r#"{"err":"bad auth","authToken":"authcookie_123"}"#;

        let parsed = parse_pipeline_message(message).unwrap();

        assert!(matches!(parsed, PipelineMessage::Error(error) if error.err == "bad auth"));
    }
}
