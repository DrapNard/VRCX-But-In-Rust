mod cache;
mod reducer;
mod state;

pub use cache::{CacheConfig, LocalCache, StoreError};
pub use reducer::{EventEffect, RecentPipelineEvent};
pub use state::{AppSnapshot, FriendPresence, SessionMetadata, WebSocketStatus};

use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{RwLock, watch};

use crate::{
    models::{friend::Friend, notification::NotificationV2},
    websocket::PipelineEvent,
};

#[derive(Clone)]
pub struct AppStore {
    cache: LocalCache,
    state: Arc<RwLock<AppSnapshot>>,
    snapshots: watch::Sender<AppSnapshot>,
}

impl AppStore {
    pub fn open(path: impl AsRef<Path>, config: CacheConfig) -> Result<Self, StoreError> {
        let cache = LocalCache::open(path, config)?;
        let initial = cache.load_snapshot()?;
        let (snapshots, _) = watch::channel(initial.clone());

        Ok(Self {
            cache,
            state: Arc::new(RwLock::new(initial)),
            snapshots,
        })
    }

    pub fn cache(&self) -> &LocalCache {
        &self.cache
    }

    pub fn subscribe(&self) -> watch::Receiver<AppSnapshot> {
        self.snapshots.subscribe()
    }

    pub async fn snapshot(&self) -> AppSnapshot {
        self.state.read().await.clone()
    }

    pub async fn set_session_metadata(&self, metadata: SessionMetadata) -> Result<(), StoreError> {
        self.cache.put_session_metadata(&metadata)?;
        let mut state = self.state.write().await;
        state.session = metadata;
        self.snapshots.send_replace(state.clone());
        Ok(())
    }

    pub async fn replace_friends(&self, friends: Vec<Friend>) -> Result<(), StoreError> {
        let mut presences = HashMap::with_capacity(friends.len());
        for friend in friends {
            let presence = FriendPresence {
                user_id: friend.id.clone(),
                display_name: Some(friend.display_name.clone()),
                status: friend.status.clone(),
                status_description: friend.status_description.clone(),
                avatar_url: friend_avatar_url(&friend),
                trust_rank: trust_rank_from_tags(&friend.tags),
                online: friend
                    .status
                    .as_deref()
                    .is_some_and(|status| status != "offline"),
                active: false,
                platform: friend.platform.clone(),
                location: friend.location.clone(),
                traveling_to_location: None,
                world_id: friend
                    .location
                    .as_deref()
                    .and_then(|location| location.split(':').next())
                    .filter(|id| id.starts_with("wrld_"))
                    .map(str::to_string),
                can_request_invite: false,
            };
            self.cache.put_friend(friend).await?;
            presences.insert(presence.user_id.clone(), presence);
        }
        let mut state = self.state.write().await;
        state.friends = presences;
        self.snapshots.send_replace(state.clone());
        Ok(())
    }

    pub async fn replace_notifications(
        &self,
        notifications: Vec<NotificationV2>,
    ) -> Result<(), StoreError> {
        let mut values = HashMap::with_capacity(notifications.len());
        for notification in notifications {
            let id = notification.id.clone();
            let value = serde_json::to_value(notification)?;
            self.cache.put_notification(&id, value.clone()).await?;
            values.insert(id, value);
        }
        let mut state = self.state.write().await;
        state.notifications = values;
        self.snapshots.send_replace(state.clone());
        Ok(())
    }

    pub async fn touch_last_sync(&self) -> Result<(), StoreError> {
        let mut metadata = self.state.read().await.session.clone();
        metadata.last_sync_unix_ms = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
        self.set_session_metadata(metadata).await
    }

    pub async fn set_websocket_status(
        &self,
        websocket_status: WebSocketStatus,
    ) -> Result<(), StoreError> {
        let mut metadata = self.state.read().await.session.clone();
        metadata.websocket_status = websocket_status;
        self.set_session_metadata(metadata).await
    }

    pub async fn set_websocket_status_with_error(
        &self,
        websocket_status: WebSocketStatus,
        websocket_error: Option<String>,
    ) -> Result<(), StoreError> {
        let mut metadata = self.state.read().await.session.clone();
        metadata.websocket_status = websocket_status;
        metadata.websocket_error = websocket_error;
        self.set_session_metadata(metadata).await
    }

    pub async fn clear_session(&self) -> Result<(), StoreError> {
        self.set_session_metadata(SessionMetadata::default()).await
    }

    pub async fn apply_pipeline_event(
        &self,
        event: PipelineEvent,
    ) -> Result<EventEffect, StoreError> {
        reducer::apply(self, event).await
    }

    async fn publish(&self) {
        let snapshot = self.state.read().await.clone();
        self.snapshots.send_replace(snapshot);
    }
}

pub(crate) fn friend_avatar_url(friend: &Friend) -> Option<String> {
    [
        friend.user_icon.as_deref(),
        friend.profile_pic_override_thumbnail.as_deref(),
        friend.current_avatar_thumbnail_image_url.as_deref(),
        friend.current_avatar_image_url.as_deref(),
        friend.image_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .find(|url| !url.trim().is_empty())
    .map(str::to_string)
}

fn trust_rank_from_tags(tags: &[String]) -> Option<String> {
    [
        ("system_trust_veteran", "trusted"),
        ("system_trust_trusted", "known"),
        ("system_trust_known", "user"),
        ("system_trust_basic", "new"),
    ]
    .into_iter()
    .find_map(|(tag, rank)| {
        tags.iter()
            .any(|value| value == tag)
            .then(|| rank.to_string())
    })
    .or_else(|| Some("visitor".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friend_avatar_ignores_empty_urls_and_uses_avatar_image() {
        let friend = Friend {
            user_icon: Some(" ".to_string()),
            profile_pic_override_thumbnail: Some(String::new()),
            current_avatar_thumbnail_image_url: None,
            current_avatar_image_url: Some("https://example.test/avatar.png".to_string()),
            ..serde_json::from_value(serde_json::json!({})).unwrap()
        };

        assert_eq!(
            friend_avatar_url(&friend).as_deref(),
            Some("https://example.test/avatar.png")
        );
    }
}
