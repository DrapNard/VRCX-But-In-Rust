use super::GroupSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedGroups {
    pub groups: Vec<GroupSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitedGroups {
    pub groups: Vec<GroupSummary>,
}
