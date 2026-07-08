#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InviteMessageType {
    Message,
    Response,
    Request,
    RequestResponse,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteMessageList {
    pub messages: Vec<InviteMessage>,
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
