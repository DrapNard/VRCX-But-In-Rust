use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::RecentPipelineEvent;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebSocketStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetadata {
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub last_sync_unix_ms: Option<u64>,
    pub websocket_status: WebSocketStatus,
    pub websocket_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct FriendPresence {
    pub user_id: String,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub status_description: Option<String>,
    pub avatar_url: Option<String>,
    pub trust_rank: Option<String>,
    pub online: bool,
    pub active: bool,
    pub platform: Option<String>,
    pub location: Option<String>,
    pub traveling_to_location: Option<String>,
    pub world_id: Option<String>,
    pub can_request_invite: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub session: SessionMetadata,
    pub friends: HashMap<String, FriendPresence>,
    pub notifications: HashMap<String, Value>,
    pub recent_events: VecDeque<RecentPipelineEvent>,
}
