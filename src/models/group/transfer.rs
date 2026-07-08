#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTransferability {
    pub requirements: GroupTransferRequirements,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTransferRequirements {
    pub group_not_monetized: bool,
    pub has_vrc_plus: bool,
    pub has_verified_email: bool,
    pub target_can_own_more_groups: bool,
    pub target_is_group_member: bool,
}
