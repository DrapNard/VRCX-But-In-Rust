#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JamState {
    Upcoming,
    Active,
    Ended,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JamSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub banner_url: Option<String>,
    pub state: JamState,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

pub type Jams = crate::models::common::Paginated<JamSummary>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jam {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub banner_url: Option<String>,
    pub state: JamState,
    pub tags: Vec<String>,
    pub rules: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub voting_starts_at: Option<String>,
    pub voting_ends_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
