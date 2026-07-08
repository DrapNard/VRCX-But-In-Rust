use crate::models::users::identity;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub version: String,
    pub store_id: String,
    pub author: identity::Identity,
}
