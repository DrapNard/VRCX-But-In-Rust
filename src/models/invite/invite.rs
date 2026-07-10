use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InviteMessageType {
    Message,
    Response,
    Request,
    RequestResponse,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteMessageSlot {
    pub slot: u8,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteMessage {
    pub id: String,
    pub message_type: InviteMessageType,
    pub message: String,
    pub slot: Option<u8>,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteMessageList {
    pub messages: Vec<InviteMessage>,
}

impl<'de> Deserialize<'de> for InviteMessageList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let messages = match value {
            Value::Array(_) => serde_json::from_value(value),
            Value::Object(mut object) => serde_json::from_value(
                object
                    .remove("messages")
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            ),
            _ => return Err(serde::de::Error::custom("expected invite message list")),
        }
        .map_err(serde::de::Error::custom)?;

        Ok(Self { messages })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteWithPhoto {
    pub user_id: String,
    pub instance_id: String,
    pub message: Option<String>,
    pub slot: Option<u8>,
    pub file_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitePhotoResponse {
    pub notification_id: String,
    pub response_slot: Option<u8>,
    pub response_message: Option<String>,
    pub file_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::InviteMessageList;

    #[test]
    fn decodes_bare_invite_message_array() {
        let messages: InviteMessageList = serde_json::from_str(
            r#"[{"id":"msg_1","messageType":"message","message":"Hello","slot":0,"updatedAt":"2026-01-01T00:00:00Z"}]"#,
        )
        .unwrap();

        assert_eq!(messages.messages.len(), 1);
        assert_eq!(messages.messages[0].message, "Hello");
    }
}
