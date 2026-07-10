use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        calendar::{
            CalendarEvent, CalendarEventCreate, CalendarEventFollow, CalendarEventList,
            CalendarEventSearch, CalendarEventUpdate,
        },
        response::ApiResponse,
    },
};

impl VrcClient {
    pub async fn calendar(&self, query: &PaginationQuery) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query("calendar", query).await
    }

    pub async fn discover_calendar(
        &self,
        query: &PaginationQuery,
    ) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query("calendar/discover", query).await
    }

    pub async fn featured_calendar(
        &self,
        query: &PaginationQuery,
    ) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query("calendar/featured", query).await
    }

    pub async fn followed_calendar(
        &self,
        query: &PaginationQuery,
    ) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query("calendar/following", query).await
    }

    pub async fn search_calendar(
        &self,
        query: &CalendarEventSearch,
    ) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query("calendar/search", query).await
    }

    pub async fn group_calendar(
        &self,
        group_id: &str,
        query: &PaginationQuery,
    ) -> Result<CalendarEventList, VrcError> {
        self.get_json_with_query(&format!("calendar/{group_id}"), query)
            .await
    }

    pub async fn create_calendar_event(
        &self,
        group_id: &str,
        body: &CalendarEventCreate,
    ) -> Result<CalendarEvent, VrcError> {
        self.post_json(&format!("calendar/{group_id}/event"), body)
            .await
    }

    pub async fn next_group_calendar_event(
        &self,
        group_id: &str,
    ) -> Result<Option<CalendarEvent>, VrcError> {
        self.get_json(&format!("calendar/{group_id}/next")).await
    }

    pub async fn calendar_event(
        &self,
        group_id: &str,
        event_id: &str,
    ) -> Result<CalendarEvent, VrcError> {
        self.get_json(&format!("calendar/{group_id}/{event_id}"))
            .await
    }

    pub async fn update_calendar_event(
        &self,
        group_id: &str,
        event_id: &str,
        body: &CalendarEventUpdate,
    ) -> Result<CalendarEvent, VrcError> {
        self.put_json(&format!("calendar/{group_id}/{event_id}/event"), body)
            .await
    }

    pub async fn delete_calendar_event(
        &self,
        group_id: &str,
        event_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("calendar/{group_id}/{event_id}/event"))
            .await
    }

    pub async fn follow_calendar_event(
        &self,
        group_id: &str,
        event_id: &str,
        body: &CalendarEventFollow,
    ) -> Result<CalendarEvent, VrcError> {
        self.post_json(&format!("calendar/{group_id}/{event_id}/follow"), body)
            .await
    }

    pub async fn calendar_event_ics(
        &self,
        group_id: &str,
        event_id: &str,
    ) -> Result<String, VrcError> {
        self.get_text(&format!("calendar/{group_id}/{event_id}.ics"))
            .await
    }
}
