#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JamState {
    Upcoming,
    Active,
    Ended,
    #[serde(other)]
    Unknown,
}

impl Default for JamState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JamSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default, alias = "title")]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    #[serde(default)]
    pub state: JamState,
    #[serde(default)]
    pub starts_at: Option<String>,
    #[serde(default)]
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
