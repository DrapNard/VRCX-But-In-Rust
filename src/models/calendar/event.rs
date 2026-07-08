use crate::models::group::GroupSummary;
use crate::models::world::WorldSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventVisibility {
    Public,
    Group,
    Friends,
    Private,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventCategory {
    Event,
    Game,
    Music,
    Performance,
    Roleplay,
    Social,
    Sports,
    Talk,
    Tutorial,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventPlatform {
    Android,
    Ios,
    StandaloneWindows,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventStats {
    pub interested: u32,
    pub going: u32,
    pub invited: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventAttendance {
    pub is_following: bool,
    pub is_interested: bool,
    pub is_going: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventSummary {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub visibility: CalendarEventVisibility,
}

pub type CalendarEventList = crate::models::common::Paginated<CalendarEventSummary>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarIcs {
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub group_id: String,
    pub group: Option<GroupSummary>,
    pub author_id: String,
    pub title: String,
    pub description: String,
    pub image_id: Option<String>,
    pub image_url: Option<String>,
    pub location: Option<String>,
    pub world_id: Option<String>,
    pub world: Option<WorldSummary>,
    pub instance_id: Option<String>,
    pub category: CalendarEventCategory,
    pub platforms: Vec<CalendarEventPlatform>,
    pub tags: Vec<String>,
    pub stats: CalendarEventStats,
    pub attendance: Option<CalendarEventAttendance>,
    pub visibility: CalendarEventVisibility,
    pub is_featured: bool,
    pub is_cancelled: bool,
    pub starts_at: String,
    pub ends_at: String,
    pub created_at: String,
    pub updated_at: String,
}
