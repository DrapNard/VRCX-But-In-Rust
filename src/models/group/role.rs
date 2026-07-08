use std::collections::HashMap;

pub type GroupRoleTemplates = HashMap<String, GroupRoleTemplate>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRole {
    pub id: String,
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    pub permissions: Vec<String>,
    pub is_management_role: bool,
    pub is_self_assignable: bool,
    pub requires_purchase: bool,
    pub requires_two_factor: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRoleTemplate {
    pub name: String,
    pub description: String,
    pub base_permissions: Vec<String>,
    pub roles: GroupRoleTemplateRole,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRoleTemplateRole {
    pub name: String,
    pub description: String,
    pub base_permissions: Vec<String>,
    pub is_added_on_join: bool,
}
