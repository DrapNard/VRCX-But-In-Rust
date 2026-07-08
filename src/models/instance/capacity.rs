#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capacity {
    pub capacity: u64,
    pub recommended_capacity: u64,
    pub user_count: u64,
    pub n_users: u64,
    pub queue_size: u64,
    pub full: bool,
    pub has_capacity_for_you: bool,
}
