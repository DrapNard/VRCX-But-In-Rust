#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionList {
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub owner_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignedPermission {
    pub permission_id: String,
    pub user_id: String,
    pub assigned_at: String,
}
