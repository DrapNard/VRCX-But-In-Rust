use super::{CalendarEventCategory, CalendarEventPlatform, CalendarEventVisibility};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventCreate {
    pub title: String,
    pub description: String,
    pub image_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub location: Option<String>,
    pub world_id: Option<String>,
    pub instance_id: Option<String>,
    pub category: CalendarEventCategory,
    pub platforms: Vec<CalendarEventPlatform>,
    pub tags: Vec<String>,
    pub visibility: CalendarEventVisibility,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_id: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub location: Option<String>,
    pub world_id: Option<String>,
    pub instance_id: Option<String>,
    pub category: Option<CalendarEventCategory>,
    pub platforms: Vec<CalendarEventPlatform>,
    pub tags: Vec<String>,
    pub visibility: Option<CalendarEventVisibility>,
    pub is_cancelled: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventSearch {
    pub query: Option<String>,
    pub group_id: Option<String>,
    pub category: Option<CalendarEventCategory>,
    pub starts_after: Option<String>,
    pub starts_before: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventFollow {
    pub interested: bool,
    pub going: bool,
}
