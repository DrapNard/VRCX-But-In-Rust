use crate::models::world::WorldSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInstance {
    pub instance_id: String,
    pub location: String,
    pub member_count: u32,
    pub world: WorldSummary,
}
