use super::identity::Identity;
use crate::models::group::GroupSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Social {
    pub is_friend: bool,
    pub friend_key: String,
    pub friend_request_status: String,
    pub mutuals: Vec<Identity>,
    pub groups: Vec<GroupSummary>,
    pub note: String,
}
