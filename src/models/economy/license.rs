#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveLicense {
    pub id: String,
    pub license_group_id: String,
    pub owner_id: String,
    pub for_id: String,
    pub for_type: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub for_type: String,
    pub licenses: Vec<ActiveLicense>,
}
