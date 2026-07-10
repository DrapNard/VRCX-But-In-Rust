use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    models::friend::Friend,
    websocket::{
        PipelineEvent,
        event::{
            FriendActiveContent, FriendLocationContent, FriendOfflineContent, FriendOnlineContent,
            UserContent,
        },
    },
};

use super::{AppStore, FriendPresence, StoreError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventEffect {
    StateChanged,
    CacheInvalidated,
    RecordedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPipelineEvent {
    pub sequence: u64,
    pub received_at_unix_ms: u64,
    pub event: PipelineEvent,
}

pub(crate) async fn apply(
    store: &AppStore,
    event: PipelineEvent,
) -> Result<EventEffect, StoreError> {
    let effect = match &event {
        PipelineEvent::FriendAdd(content) | PipelineEvent::FriendUpdate(content) => {
            upsert_friend(store, content).await?;
            EventEffect::StateChanged
        }
        PipelineEvent::FriendDelete { user_id } => {
            store.cache.remove_friend(user_id).await?;
            store.state.write().await.friends.remove(user_id);
            EventEffect::StateChanged
        }
        PipelineEvent::FriendOnline(content) => {
            friend_online(store, content).await?;
            EventEffect::StateChanged
        }
        PipelineEvent::FriendActive(content) => {
            friend_active(store, content).await?;
            EventEffect::StateChanged
        }
        PipelineEvent::FriendOffline(content) => {
            friend_offline(store, content).await;
            EventEffect::StateChanged
        }
        PipelineEvent::FriendLocation(content) => {
            friend_location(store, content).await?;
            EventEffect::StateChanged
        }
        PipelineEvent::Notification(value) | PipelineEvent::NotificationV2(value) => {
            if let Some(id) = value.get("id").and_then(Value::as_str) {
                store.cache.put_notification(id, value.clone()).await?;
                store
                    .state
                    .write()
                    .await
                    .notifications
                    .insert(id.to_string(), value.clone());
                EventEffect::StateChanged
            } else {
                EventEffect::RecordedOnly
            }
        }
        PipelineEvent::SeeNotification(id) | PipelineEvent::HideNotification(id) => {
            update_notification_flag(store, id, &event).await?;
            EventEffect::StateChanged
        }
        PipelineEvent::ClearNotification => {
            let ids = store
                .state
                .read()
                .await
                .notifications
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            for id in ids {
                store.cache.remove_notification(&id).await?;
            }
            store.state.write().await.notifications.clear();
            EventEffect::StateChanged
        }
        PipelineEvent::NotificationV2Update(update) => {
            if let Some(mut value) = store
                .cache
                .notification(&update.id)
                .await?
                .map(|value| (*value).clone())
            {
                merge_json(&mut value, &update.updates);
                store
                    .cache
                    .put_notification(&update.id, value.clone())
                    .await?;
                store
                    .state
                    .write()
                    .await
                    .notifications
                    .insert(update.id.clone(), value);
                EventEffect::StateChanged
            } else {
                EventEffect::CacheInvalidated
            }
        }
        PipelineEvent::NotificationV2Delete(deleted) => {
            for id in &deleted.ids {
                store.cache.remove_notification(id).await?;
                store.state.write().await.notifications.remove(id);
            }
            EventEffect::StateChanged
        }
        PipelineEvent::ContentRefresh(_) => EventEffect::CacheInvalidated,
        _ => EventEffect::RecordedOnly,
    };

    record_event(store, event).await?;
    store.publish().await;
    Ok(effect)
}

async fn upsert_friend(store: &AppStore, content: &UserContent) -> Result<(), StoreError> {
    if let Ok(friend) = serde_json::from_value::<Friend>(content.user.clone()) {
        let presence = presence_from_friend(&friend);
        store.cache.put_friend(friend).await?;
        store
            .state
            .write()
            .await
            .friends
            .insert(content.user_id.clone(), presence);
    } else {
        let mut state = store.state.write().await;
        let presence = state
            .friends
            .entry(content.user_id.clone())
            .or_insert_with(|| empty_presence(&content.user_id));
        apply_user_fields(presence, &content.user);
    }
    Ok(())
}

async fn friend_online(store: &AppStore, content: &FriendOnlineContent) -> Result<(), StoreError> {
    cache_embedded_friend(store, &content.user).await?;
    let mut state = store.state.write().await;
    let presence = state
        .friends
        .entry(content.user_id.clone())
        .or_insert_with(|| empty_presence(&content.user_id));
    apply_user_fields(presence, &content.user);
    presence.online = true;
    presence.active = true;
    presence.platform = Some(content.platform.clone());
    presence.location = Some(content.location.clone());
    presence.can_request_invite = content.can_request_invite;
    Ok(())
}

async fn friend_active(store: &AppStore, content: &FriendActiveContent) -> Result<(), StoreError> {
    cache_embedded_friend(store, &content.user).await?;
    let mut state = store.state.write().await;
    let presence = state
        .friends
        .entry(content.user_id.clone())
        .or_insert_with(|| empty_presence(&content.user_id));
    apply_user_fields(presence, &content.user);
    presence.online = true;
    presence.active = true;
    presence.platform = Some(content.platform.clone());
    Ok(())
}

async fn friend_offline(store: &AppStore, content: &FriendOfflineContent) {
    let mut state = store.state.write().await;
    let presence = state
        .friends
        .entry(content.user_id.clone())
        .or_insert_with(|| empty_presence(&content.user_id));
    presence.online = false;
    presence.active = false;
    presence.platform = Some(content.platform.clone());
    presence.location = None;
    presence.traveling_to_location = None;
    presence.world_id = None;
    presence.can_request_invite = false;
}

async fn friend_location(
    store: &AppStore,
    content: &FriendLocationContent,
) -> Result<(), StoreError> {
    cache_embedded_friend(store, &content.user).await?;
    let mut state = store.state.write().await;
    let presence = state
        .friends
        .entry(content.user_id.clone())
        .or_insert_with(|| empty_presence(&content.user_id));
    apply_user_fields(presence, &content.user);
    presence.online = true;
    presence.location = Some(content.location.clone());
    presence.traveling_to_location = non_empty(&content.traveling_to_location);
    presence.world_id = non_empty(&content.world_id);
    presence.can_request_invite = content.can_request_invite;
    Ok(())
}

async fn cache_embedded_friend(store: &AppStore, value: &Value) -> Result<(), StoreError> {
    if let Ok(friend) = serde_json::from_value::<Friend>(value.clone()) {
        store.cache.put_friend(friend).await?;
    }
    Ok(())
}

async fn update_notification_flag(
    store: &AppStore,
    id: &str,
    event: &PipelineEvent,
) -> Result<(), StoreError> {
    let cached = store
        .cache
        .notification(id)
        .await?
        .map(|value| (*value).clone());
    let Some(mut value) = cached else {
        return Ok(());
    };
    if let Some(object) = value.as_object_mut() {
        match event {
            PipelineEvent::SeeNotification(_) => {
                object.insert("seen".to_string(), Value::Bool(true));
            }
            PipelineEvent::HideNotification(_) => {
                object.insert("status".to_string(), Value::String("hidden".to_string()));
            }
            _ => {}
        }
    }
    store.cache.put_notification(id, value.clone()).await?;
    store
        .state
        .write()
        .await
        .notifications
        .insert(id.to_string(), value);
    Ok(())
}

async fn record_event(store: &AppStore, event: PipelineEvent) -> Result<(), StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut state = store.state.write().await;
    let sequence = state
        .recent_events
        .back()
        .map_or(now, |last| last.sequence.saturating_add(1).max(now));
    let recent = RecentPipelineEvent {
        sequence,
        received_at_unix_ms: now,
        event,
    };
    store.cache.append_event(&recent)?;
    state.recent_events.push_back(recent);
    while state.recent_events.len() > store.cache.max_recent_events {
        state.recent_events.pop_front();
    }
    Ok(())
}

fn empty_presence(user_id: &str) -> FriendPresence {
    FriendPresence {
        user_id: user_id.to_string(),
        ..FriendPresence::default()
    }
}

fn presence_from_friend(friend: &Friend) -> FriendPresence {
    FriendPresence {
        user_id: friend.id.clone(),
        display_name: Some(friend.display_name.clone()),
        status: friend.status.clone(),
        avatar_url: super::friend_avatar_url(friend),
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
    }
}

fn apply_user_fields(presence: &mut FriendPresence, user: &Value) {
    if let Some(display_name) = user.get("displayName").and_then(Value::as_str) {
        presence.display_name = Some(display_name.to_string());
    }
    if let Some(platform) = user.get("platform").and_then(Value::as_str) {
        presence.platform = Some(platform.to_string());
    }
    if let Some(status) = user.get("status").and_then(Value::as_str) {
        presence.status = Some(status.to_string());
    }
    presence.avatar_url = [
        "userIcon",
        "profilePicOverrideThumbnail",
        "currentAvatarThumbnailImageUrl",
        "currentAvatarImageUrl",
        "imageUrl",
    ]
    .into_iter()
    .find_map(|key| {
        user.get(key)
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(str::to_string)
    })
    .or_else(|| presence.avatar_url.clone());
    if let Some(tags) = user.get("tags").and_then(Value::as_array) {
        let tags = tags
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        presence.trust_rank = trust_rank_from_tags(&tags);
    }
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

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn merge_json(target: &mut Value, updates: &Value) {
    match (target, updates) {
        (Value::Object(target), Value::Object(updates)) => {
            for (key, value) in updates {
                merge_json(target.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (target, updates) => *target = updates.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::{
        store::CacheConfig,
        websocket::event::{
            FriendLocationContent, FriendOfflineContent, FriendOnlineContent, NotificationV2Delete,
        },
    };

    fn database_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vrcx-rust-{name}-{}-{}.redb",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn friend_presence_tracks_online_location_and_offline() {
        let path = database_path("presence");
        let store = AppStore::open(&path, CacheConfig::default()).unwrap();

        store
            .apply_pipeline_event(PipelineEvent::FriendOnline(FriendOnlineContent {
                user_id: "usr_1".to_string(),
                platform: "standalonewindows".to_string(),
                location: "private".to_string(),
                can_request_invite: true,
                user: json!({"id": "usr_1", "displayName": "Ada"}),
            }))
            .await
            .unwrap();
        store
            .apply_pipeline_event(PipelineEvent::FriendLocation(FriendLocationContent {
                user_id: "usr_1".to_string(),
                location: "wrld_1:123".to_string(),
                traveling_to_location: String::new(),
                world_id: "wrld_1".to_string(),
                can_request_invite: false,
                user: json!({"id": "usr_1", "displayName": "Ada"}),
            }))
            .await
            .unwrap();

        let presence = store.snapshot().await.friends["usr_1"].clone();
        assert!(presence.online);
        assert_eq!(presence.display_name.as_deref(), Some("Ada"));
        assert_eq!(presence.world_id.as_deref(), Some("wrld_1"));

        store
            .apply_pipeline_event(PipelineEvent::FriendOffline(FriendOfflineContent {
                user_id: "usr_1".to_string(),
                platform: "standalonewindows".to_string(),
            }))
            .await
            .unwrap();
        let presence = store.snapshot().await.friends["usr_1"].clone();
        assert!(!presence.online);
        assert!(presence.location.is_none());
        assert!(presence.world_id.is_none());

        drop(store);
        std::fs::remove_file(path).ok();
    }

    #[tokio::test]
    async fn notifications_and_recent_events_survive_reopen() {
        let path = database_path("persistence");
        let mut config = CacheConfig::default();
        config.max_recent_events = 2;

        {
            let store = AppStore::open(&path, config.clone()).unwrap();
            store
                .apply_pipeline_event(PipelineEvent::NotificationV2(json!({
                    "id": "not_1",
                    "message": "hello",
                    "seen": false
                })))
                .await
                .unwrap();
            store
                .apply_pipeline_event(PipelineEvent::SeeNotification("not_1".to_string()))
                .await
                .unwrap();
            store
                .apply_pipeline_event(PipelineEvent::Unknown {
                    event_type: "future-event".to_string(),
                    content: Value::Null,
                })
                .await
                .unwrap();
        }

        let reopened = AppStore::open(&path, config).unwrap();
        let snapshot = reopened.snapshot().await;
        assert_eq!(snapshot.recent_events.len(), 2);
        assert_eq!(snapshot.notifications["not_1"]["seen"], true);

        reopened
            .apply_pipeline_event(PipelineEvent::NotificationV2Delete(NotificationV2Delete {
                ids: vec!["not_1".to_string()],
                version: 2,
            }))
            .await
            .unwrap();
        assert!(reopened.snapshot().await.notifications.is_empty());

        drop(reopened);
        std::fs::remove_file(path).ok();
    }
}
