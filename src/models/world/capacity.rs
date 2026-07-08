#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capacity {
    pub capacity: u16,
    pub recommended_capacity: u16,
}
