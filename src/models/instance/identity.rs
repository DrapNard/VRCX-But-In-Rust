use crate::models::world::WorldSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub instance_id: String,
    pub world: WorldSummary,
    pub name: String,
    pub display_name: String,
    pub short_name: String,
    pub secure_name: String,
    pub client_number: u8,
}
