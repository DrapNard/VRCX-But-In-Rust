#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPermission {
    pub name: String,
    pub display_name: String,
    pub help: String,
    pub allowed_to_add: bool,
    pub is_management_permission: bool,
}
