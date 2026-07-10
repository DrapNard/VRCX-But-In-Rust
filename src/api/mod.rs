pub mod auth;
pub mod avatars;
pub mod calendar;
pub mod economy;
pub mod favorites;
pub mod files;
pub mod friends;
pub mod groups;
pub mod instances;
pub mod inventory;
pub mod invites;
pub mod jams;
pub mod moderation;
pub mod notifications;
pub mod prints;
pub mod props;
pub mod system;
pub mod users;
pub mod worlds;

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationQuery {
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl PaginationQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
}
