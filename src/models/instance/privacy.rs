use crate::models::group::GroupSummary;
use crate::models::users::UserSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstanceType {
    Public,
    FriendPlus,
    Friend,
    InvitePlus,
    Invite,
    GroupPublic,
    GroupPlus,
    GroupMember,
    Private,
    Hidden,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Privacy {
    pub kind: String,
    pub owner: Option<UserSummary>,
    pub group: Option<GroupSummary>,
    pub r#type: InstanceType,
    pub age_gate: bool,
    pub nonce: bool,
}
