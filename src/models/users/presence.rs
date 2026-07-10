use crate::models::instance::InstanceSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Traveling {
    None,
    Offline,
    Traveling,
    Private,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presence {
    pub instance: Option<InstanceSummary>,
    pub traveling_to_instance: Option<InstanceSummary>,
    pub travels: Vec<Traveling>,
    pub platform: String,
    pub last_platform: String,
}
