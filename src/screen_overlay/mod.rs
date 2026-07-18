//! Lightweight, event-driven on-screen notifications.
//!
//! No window and no timer exist while the overlay is idle. A small transparent,
//! click-through window is created only while one or more toasts are visible.

mod config;

pub use config::ScreenOverlayConfig;

use crate::{
    store::{FriendPresence, RecentPipelineEvent},
    websocket::PipelineEvent,
};
use iced::{
    Border, Color, ContentFit, Element, Fill, Length, Point, Size,
    widget::{Space, column, container, image, row, rule, text},
    window,
};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::PathBuf,
    time::Duration,
};

const TOAST_HEIGHT: f32 = 92.0;

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
    pub opacity: f32,
    pub duration: Duration,
    pub avatar_url: Option<String>,
    pub world_image_url: Option<String>,
    world_id: Option<String>,
    show_world_picture: bool,
}

#[derive(Debug)]
pub struct ScreenOverlay {
    pub config: ScreenOverlayConfig,
    pub config_path: PathBuf,
    pub window_id: Option<window::Id>,
    toasts: VecDeque<Toast>,
    last_sequence: Option<u64>,
    next_toast_id: u64,
    friend_names: HashMap<String, String>,
    friend_avatars: HashMap<String, String>,
    world_names: HashMap<String, String>,
    world_images: HashMap<String, String>,
}

impl ScreenOverlay {
    pub fn load() -> Self {
        let config_path = config_path();
        let config = match fs::read_to_string(&config_path) {
            Ok(raw) => match serde_json::from_str::<ScreenOverlayConfig>(&raw) {
                Ok(config) => config.sanitized(),
                Err(error) => {
                    tracing::warn!(path = %config_path.display(), %error, "invalid screen overlay config; using defaults");
                    ScreenOverlayConfig::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let config = ScreenOverlayConfig::default();
                if let Some(parent) = config_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(json) = serde_json::to_string_pretty(&config)
                    && let Err(error) = fs::write(&config_path, json)
                {
                    tracing::warn!(path = %config_path.display(), %error, "cannot create screen overlay config");
                }
                config
            }
            Err(error) => {
                tracing::warn!(path = %config_path.display(), %error, "cannot read screen overlay config; using defaults");
                ScreenOverlayConfig::default()
            }
        };

        Self::from_config(config, config_path)
    }

    fn from_config(config: ScreenOverlayConfig, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
            window_id: None,
            toasts: VecDeque::new(),
            last_sequence: None,
            next_toast_id: 1,
            friend_names: HashMap::new(),
            friend_avatars: HashMap::new(),
            world_names: HashMap::new(),
            world_images: HashMap::new(),
        }
    }

    /// Returns newly accepted toasts. The first snapshot only establishes a watermark,
    /// so cached events never produce a burst when the application starts.
    pub fn ingest<'a>(
        &'a mut self,
        events: impl IntoIterator<Item = &'a RecentPipelineEvent>,
        friends: &HashMap<String, FriendPresence>,
        favorite_friend_ids: &HashSet<String>,
        world_names: &HashMap<String, String>,
        world_images: &HashMap<String, String>,
    ) -> Vec<Toast> {
        self.update_friend_cache(friends);
        self.world_names.extend(world_names.clone());
        self.world_images.extend(world_images.clone());
        let events = events.into_iter().collect::<Vec<_>>();
        let newest = events.iter().map(|event| event.sequence).max();
        let Some(watermark) = self.last_sequence else {
            self.last_sequence = newest;
            return Vec::new();
        };

        let mut accepted = Vec::new();
        if self.config.enabled {
            for recent in events
                .into_iter()
                .filter(|event| event.sequence > watermark)
            {
                if let Some(toast) = self.toast_for(recent, favorite_friend_ids) {
                    accepted.push(toast.clone());
                    self.toasts.push_back(toast);
                    while self.toasts.len() > self.config.max_visible {
                        self.toasts.pop_front();
                    }
                }
            }
        }
        self.last_sequence = newest.or(self.last_sequence);
        accepted
    }

    pub fn reset(&mut self) {
        self.toasts.clear();
        self.last_sequence = None;
    }

    pub fn save_config(&self) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let json = serde_json::to_string_pretty(&self.config).map_err(|error| error.to_string())?;
        fs::write(&self.config_path, json).map_err(|error| error.to_string())
    }

    pub fn preview(&mut self, event: &str) -> Toast {
        let id = self.next_toast_id;
        self.next_toast_id = self.next_toast_id.wrapping_add(1).max(1);
        let rule = self.config.rules.get(event).cloned().unwrap_or_default();
        let toast = Toast {
            id,
            title: render_preview_template(&rule.title, event),
            body: render_preview_template(&rule.body, event),
            accent: parse_color(&rule.accent).unwrap_or(Color::from_rgb8(108, 140, 255)),
            background: parse_color(&self.config.background)
                .unwrap_or(Color::from_rgb8(17, 19, 30)),
            foreground: parse_color(&self.config.foreground).unwrap_or(Color::WHITE),
            opacity: rule.opacity.unwrap_or(self.config.default_opacity),
            duration: Duration::from_secs_f32(
                rule.duration_seconds
                    .unwrap_or(self.config.default_duration_seconds),
            ),
            avatar_url: None,
            world_image_url: None,
            world_id: None,
            show_world_picture: false,
        };
        self.toasts.push_back(toast.clone());
        while self.toasts.len() > self.config.max_visible {
            self.toasts.pop_front();
        }
        toast
    }

    pub fn expire(&mut self, id: u64) -> bool {
        if let Some(index) = self.toasts.iter().position(|toast| toast.id == id) {
            self.toasts.remove(index);
        }
        self.toasts.is_empty()
    }

    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    pub fn image_urls(&self) -> impl Iterator<Item = &String> {
        self.toasts.iter().flat_map(|toast| {
            [toast.avatar_url.as_ref(), toast.world_image_url.as_ref()]
                .into_iter()
                .flatten()
        })
    }

    pub fn resolve_world(&mut self, world_id: &str, name: &str, image_url: Option<&str>) {
        self.world_names.insert(world_id.into(), name.into());
        if let Some(url) = image_url {
            self.world_images.insert(world_id.into(), url.into());
        }
        for toast in self
            .toasts
            .iter_mut()
            .filter(|toast| toast.world_id.as_deref() == Some(world_id))
        {
            toast.title = toast.title.replace(world_id, name);
            toast.title = toast.title.replace("Monde VRChat", name);
            toast.body = toast
                .body
                .replace(world_id, name)
                .replace("Monde VRChat", name);
            if toast.show_world_picture {
                toast.world_image_url = image_url.map(str::to_string);
            }
        }
    }

    pub fn window_settings(&self) -> window::Settings {
        let count = self.toasts.len().max(1) as f32;
        let height = self.config.margin as f32 * 2.0
            + count * TOAST_HEIGHT
            + (count - 1.0) * self.config.spacing as f32;
        window::Settings {
            size: Size::new(self.config.width as f32, height),
            position: window::Position::SpecificWith(top_right),
            resizable: false,
            closeable: false,
            minimizable: false,
            decorations: false,
            transparent: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            ..Default::default()
        }
    }

    pub fn view<'a, Message: 'static>(
        &'a self,
        thumbnails: &'a HashMap<String, image::Handle>,
    ) -> Element<'a, Message> {
        let content = self.toasts.iter().fold(
            column![].spacing(self.config.spacing as f32),
            |content, toast| {
                let accent = toast.accent;
                let background = Color {
                    a: toast.opacity,
                    ..toast.background
                };
                let foreground = toast.foreground;
                let avatar: Element<'_, Message> = toast
                    .avatar_url
                    .as_ref()
                    .and_then(|url| thumbnails.get(url))
                    .map_or_else(
                        || Space::new().width(0).into(),
                        |handle| {
                            image(handle.clone())
                                .width(40)
                                .height(40)
                                .content_fit(ContentFit::Cover)
                                .into()
                        },
                    );
                let world_picture: Element<'_, Message> = toast
                    .world_image_url
                    .as_ref()
                    .and_then(|url| thumbnails.get(url))
                    .map_or_else(
                        || Space::new().width(0).into(),
                        |handle| {
                            image(handle.clone())
                                .width(40)
                                .height(40)
                                .content_fit(ContentFit::Cover)
                                .into()
                        },
                    );
                let card = row![
                    rule::vertical(4).style(move |_| rule::Style {
                        color: accent,
                        radius: Border::default().radius,
                        fill_mode: rule::FillMode::Full,
                        snap: true,
                    }),
                    avatar,
                    world_picture,
                    column![
                        text(&toast.title).size(16).color(foreground),
                        text(&toast.body).size(13).color(Color {
                            a: 0.82,
                            ..foreground
                        }),
                    ]
                    .spacing(5)
                    .width(Fill),
                ]
                .spacing(12);
                content.push(
                    container(card)
                        .height(TOAST_HEIGHT)
                        .width(Fill)
                        .padding([14, 16])
                        .style(move |_| {
                            container::Style::default()
                                .background(background)
                                .border(Border::default().rounded(10))
                        }),
                )
            },
        );

        container(content)
            .padding(self.config.margin)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn toast_for(
        &mut self,
        recent: &RecentPipelineEvent,
        favorite_friend_ids: &HashSet<String>,
    ) -> Option<Toast> {
        let mut fields = event_fields(&recent.event);
        let rule = self.config.rules.get(fields.key)?.clone();
        let rejected_user = fields.user_id.is_some_and(|user_id| {
            if !rule.selected_users.is_empty() {
                !rule.selected_users.contains(user_id)
            } else {
                rule.favorites_only && !favorite_friend_ids.contains(user_id)
            }
        });
        if !rule.enabled || rejected_user {
            return None;
        }

        if let Some(user_id) = fields.user_id {
            if fields.name.is_empty() || fields.name == user_id {
                fields.name = self
                    .friend_names
                    .get(user_id)
                    .cloned()
                    .unwrap_or_else(|| "Utilisateur VRChat".to_string());
            }
        }
        let world_id = fields.world_id.map(str::to_string);
        if let Some(world_id) = world_id.as_deref()
            && let Some(name) = self.world_names.get(world_id)
        {
            fields.location = name.clone();
            fields.message = fields.message.replace(world_id, name);
        } else if let Some(world_id) = world_id.as_deref() {
            fields.location = "Monde VRChat".to_string();
            fields.message = fields.message.replace(world_id, "Monde VRChat");
        }

        let id = self.next_toast_id;
        self.next_toast_id = self.next_toast_id.wrapping_add(1).max(1);
        Some(Toast {
            id,
            title: render_template(&rule.title, &fields),
            body: render_template(&rule.body, &fields),
            accent: parse_color(&rule.accent).unwrap_or(Color::from_rgb8(108, 140, 255)),
            background: parse_color(&self.config.background)
                .unwrap_or(Color::from_rgb8(17, 19, 30)),
            foreground: parse_color(&self.config.foreground).unwrap_or(Color::WHITE),
            opacity: rule.opacity.unwrap_or(self.config.default_opacity),
            duration: Duration::from_secs_f32(
                rule.duration_seconds
                    .unwrap_or(self.config.default_duration_seconds),
            ),
            avatar_url: rule
                .show_profile_picture
                .then(|| {
                    fields
                        .user_id
                        .and_then(|id| self.friend_avatars.get(id).cloned())
                })
                .flatten(),
            world_image_url: rule
                .show_world_picture
                .then(|| {
                    world_id
                        .as_deref()
                        .and_then(|id| self.world_images.get(id).cloned())
                })
                .flatten(),
            world_id,
            show_world_picture: rule.show_world_picture,
        })
    }

    fn update_friend_cache(&mut self, friends: &HashMap<String, FriendPresence>) {
        for friend in friends.values() {
            if let Some(name) = friend.display_name.as_ref().filter(|name| !name.is_empty()) {
                self.friend_names
                    .insert(friend.user_id.clone(), name.clone());
            }
            if let Some(url) = friend.avatar_url.as_ref().filter(|url| !url.is_empty()) {
                self.friend_avatars
                    .insert(friend.user_id.clone(), url.clone());
            }
        }
    }
}

fn config_path() -> PathBuf {
    directories::ProjectDirs::from("dev", "vrcx-rs", "VRCX Rust")
        .map(|dirs| dirs.config_dir().join("screen-overlay.json"))
        .unwrap_or_else(|| PathBuf::from("screen-overlay.json"))
}

fn top_right(window: Size, monitor: Size) -> Point {
    Point::new((monitor.width - window.width - 12.0).max(0.0), 12.0)
}

struct EventFields<'a> {
    key: &'static str,
    event: &'static str,
    name: String,
    user_id: Option<&'a str>,
    platform: &'a str,
    location: String,
    world_id: Option<&'a str>,
    message: String,
}

fn event_fields(event: &PipelineEvent) -> EventFields<'_> {
    let empty = "";
    match event {
        PipelineEvent::FriendOnline(content) => EventFields {
            key: "friend-online",
            event: "Friend online",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: &content.platform,
            location: world_id_from_location(&content.location)
                .unwrap_or(&content.location)
                .to_string(),
            world_id: world_id_from_location(&content.location),
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendOffline(content) => EventFields {
            key: "friend-offline",
            event: "Friend offline",
            name: content.user_id.clone(),
            user_id: Some(&content.user_id),
            platform: &content.platform,
            location: String::new(),
            world_id: None,
            message: content.user_id.clone(),
        },
        PipelineEvent::FriendActive(content) => EventFields {
            key: "friend-active",
            event: "Friend active",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: &content.platform,
            location: String::new(),
            world_id: None,
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendLocation(content) => EventFields {
            key: "friend-location",
            event: "Friend location",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: empty,
            location: world_id_from_location(&content.location)
                .unwrap_or(&content.location)
                .to_string(),
            world_id: world_id_from_location(&content.location),
            message: world_id_from_location(&content.location)
                .unwrap_or(&content.location)
                .to_string(),
        },
        PipelineEvent::FriendAdd(content) => EventFields {
            key: "friend-add",
            event: "Friend added",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: empty,
            location: String::new(),
            world_id: None,
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendDelete { user_id } => EventFields {
            key: "friend-delete",
            event: "Friend removed",
            name: user_id.clone(),
            user_id: Some(user_id),
            platform: empty,
            location: String::new(),
            world_id: None,
            message: user_id.clone(),
        },
        PipelineEvent::Notification(value) | PipelineEvent::NotificationV2(value) => EventFields {
            key: "notification",
            event: "Notification",
            name: json_string(value, "senderUsername"),
            user_id: value.get("senderUserId").and_then(Value::as_str),
            platform: empty,
            location: String::new(),
            world_id: None,
            message: json_string(value, "message"),
        },
        PipelineEvent::InstanceQueueReady(content) => EventFields {
            key: "instance-queue-ready",
            event: "Instance ready",
            name: String::new(),
            user_id: None,
            platform: empty,
            location: world_id_from_location(&content.instance_location)
                .unwrap_or(&content.instance_location)
                .to_string(),
            world_id: world_id_from_location(&content.instance_location),
            message: world_id_from_location(&content.instance_location)
                .unwrap_or(&content.instance_location)
                .to_string(),
        },
        PipelineEvent::InstanceQueueJoined(content) => EventFields {
            key: "instance-queue-joined",
            event: "Instance queue",
            name: String::new(),
            user_id: None,
            platform: empty,
            location: world_id_from_location(&content.instance_location)
                .unwrap_or(&content.instance_location)
                .to_string(),
            world_id: world_id_from_location(&content.instance_location),
            message: format!("Position {}", content.position),
        },
        PipelineEvent::UserLocation(content) => EventFields {
            key: "user-location",
            event: "User location",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: empty,
            location: world_id_from_location(&content.location)
                .unwrap_or(&content.location)
                .to_string(),
            world_id: world_id_from_location(&content.location),
            message: world_id_from_location(&content.location)
                .unwrap_or(&content.location)
                .to_string(),
        },
        other => EventFields {
            key: event_key(other),
            event: event_key(other),
            name: String::new(),
            user_id: None,
            platform: empty,
            location: String::new(),
            world_id: None,
            message: event_key(other).to_string(),
        },
    }
}

fn event_key(event: &PipelineEvent) -> &'static str {
    match event {
        PipelineEvent::FriendUpdate(_) => "friend-update",
        PipelineEvent::UserUpdate(_) => "user-update",
        PipelineEvent::UserLocation(_) => "user-location",
        PipelineEvent::ResponseNotification(_) => "response-notification",
        PipelineEvent::SeeNotification(_) => "see-notification",
        PipelineEvent::HideNotification(_) => "hide-notification",
        PipelineEvent::ClearNotification => "clear-notification",
        PipelineEvent::NotificationV2Update(_) => "notification-v2-update",
        PipelineEvent::NotificationV2Delete(_) => "notification-v2-delete",
        PipelineEvent::UserBadgeAssigned(_) => "user-badge-assigned",
        PipelineEvent::UserBadgeUnassigned(_) => "user-badge-unassigned",
        PipelineEvent::ContentRefresh(_) => "content-refresh",
        PipelineEvent::EconomyUpdate(_) => "economy-update",
        PipelineEvent::ModifiedImageUpdate(_) => "modified-image-update",
        PipelineEvent::GroupJoined(_) => "group-joined",
        PipelineEvent::GroupLeft(_) => "group-left",
        PipelineEvent::GroupMemberUpdated(_) => "group-member-updated",
        PipelineEvent::GroupRoleUpdated(_) => "group-role-updated",
        PipelineEvent::Unknown { .. } => "unknown",
        _ => "event",
    }
}

fn world_id_from_location(location: &str) -> Option<&str> {
    let world_id = location.split(':').next().unwrap_or(location);
    world_id.starts_with("wrld_").then_some(world_id)
}

fn render_template(template: &str, fields: &EventFields<'_>) -> String {
    template
        .replace("{event}", fields.event)
        .replace("{name}", &fields.name)
        .replace("{user_id}", fields.user_id.unwrap_or_default())
        .replace("{platform}", fields.platform)
        .replace("{location}", &fields.location)
        .replace("{message}", &fields.message)
}

fn render_preview_template(template: &str, event: &str) -> String {
    template
        .replace("{event}", event)
        .replace("{name}", "Utilisateur test")
        .replace("{user_id}", "usr_test")
        .replace("{platform}", "PC")
        .replace("{location}", "Monde de test")
        .replace("{message}", "Ceci est une notification de test")
}

fn display_name(value: &Value, fallback: &str) -> String {
    value
        .get("displayName")
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn json_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim().strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    Some(Color::from_rgb8(
        u8::from_str_radix(&value[0..2], 16).ok()?,
        u8::from_str_radix(&value[2..4], 16).ok()?,
        u8::from_str_radix(&value[4..6], 16).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::event::{FriendOfflineContent, FriendOnlineContent};
    use std::collections::{HashMap, HashSet};

    fn active_config() -> ScreenOverlayConfig {
        let mut config = ScreenOverlayConfig::default();
        config.enabled = true;
        for rule in config.rules.values_mut() {
            rule.favorites_only = false;
        }
        config
    }

    fn online(sequence: u64, user_id: &str) -> RecentPipelineEvent {
        RecentPipelineEvent {
            sequence,
            received_at_unix_ms: sequence,
            event: PipelineEvent::FriendOnline(FriendOnlineContent {
                user_id: user_id.into(),
                user: serde_json::json!({"displayName": "Alice"}),
                platform: "standalonewindows".into(),
                location: "wrld_test:1".into(),
                can_request_invite: true,
            }),
        }
    }

    #[test]
    fn cached_events_are_ignored_and_new_events_are_emitted_once() {
        let mut overlay = ScreenOverlay::from_config(active_config(), PathBuf::from("test.json"));
        let friends = HashMap::new();
        let favorites = HashSet::new();
        let worlds = HashMap::new();
        assert!(
            overlay
                .ingest(
                    [&online(1, "usr_a")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .is_empty()
        );
        assert_eq!(
            overlay
                .ingest(
                    [&online(1, "usr_a"), &online(2, "usr_a")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .len(),
            1
        );
        assert!(
            overlay
                .ingest(
                    [&online(2, "usr_a")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .is_empty()
        );
    }

    #[test]
    fn selected_user_filter_is_applied() {
        let mut overlay = ScreenOverlay::from_config(active_config(), PathBuf::from("test.json"));
        overlay
            .config
            .rules
            .get_mut("friend-online")
            .unwrap()
            .selected_users = ["usr_wanted".to_string()].into_iter().collect();
        let friends = HashMap::new();
        let favorites = HashSet::new();
        let worlds = HashMap::new();
        overlay.ingest(
            [&online(1, "usr_wanted")],
            &friends,
            &favorites,
            &worlds,
            &worlds,
        );
        assert!(
            overlay
                .ingest(
                    [&online(2, "usr_other")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .is_empty()
        );
        assert_eq!(
            overlay
                .ingest(
                    [&online(3, "usr_wanted")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .len(),
            1
        );
    }

    #[test]
    fn preview_works_even_when_live_overlay_is_disabled() {
        let mut config = ScreenOverlayConfig::default();
        config.enabled = true;
        config.enabled = false;
        let mut overlay = ScreenOverlay::from_config(config, PathBuf::from("test.json"));
        let toast = overlay.preview("friend-online");
        assert!(overlay.has_toasts());
        assert!(toast.title.contains("Utilisateur test"));
        assert!(overlay.expire(toast.id));
    }

    #[test]
    fn defaults_are_disabled_and_only_accept_favorite_friends() {
        let config = ScreenOverlayConfig::default();
        assert!(!config.enabled);
        assert!(config.rules["friend-online"].favorites_only);

        let mut config = config;
        config.enabled = true;
        let mut overlay = ScreenOverlay::from_config(config, PathBuf::from("test.json"));
        let friends = HashMap::new();
        let favorites = HashSet::from(["usr_favorite".to_string()]);
        let worlds = HashMap::new();
        overlay.ingest(
            [&online(1, "usr_other")],
            &friends,
            &favorites,
            &worlds,
            &worlds,
        );
        assert!(
            overlay
                .ingest(
                    [&online(2, "usr_other")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .is_empty()
        );
        assert_eq!(
            overlay
                .ingest(
                    [&online(3, "usr_favorite")],
                    &friends,
                    &favorites,
                    &worlds,
                    &worlds,
                )
                .len(),
            1
        );
    }

    #[test]
    fn id_only_events_use_cached_name_and_profile_picture() {
        let mut config = ScreenOverlayConfig::default();
        config.enabled = true;
        let rule = config.rules.get_mut("friend-offline").unwrap();
        rule.enabled = true;
        rule.title = "{name} est hors ligne".into();
        let mut overlay = ScreenOverlay::from_config(config, PathBuf::from("test.json"));
        let friends = HashMap::from([(
            "usr_bob".to_string(),
            FriendPresence {
                user_id: "usr_bob".into(),
                display_name: Some("Bob".into()),
                avatar_url: Some("https://example.com/bob.png".into()),
                ..FriendPresence::default()
            },
        )]);
        let worlds = HashMap::new();
        let favorites = HashSet::from(["usr_bob".to_string()]);
        overlay.ingest(
            [&online(1, "usr_bob")],
            &friends,
            &favorites,
            &worlds,
            &worlds,
        );
        let offline = RecentPipelineEvent {
            sequence: 2,
            received_at_unix_ms: 2,
            event: PipelineEvent::FriendOffline(FriendOfflineContent {
                user_id: "usr_bob".into(),
                platform: "standalonewindows".into(),
            }),
        };
        let toasts = overlay.ingest([&offline], &HashMap::new(), &favorites, &worlds, &worlds);
        assert_eq!(toasts[0].title, "Bob est hors ligne");
        assert_eq!(
            toasts[0].avatar_url.as_deref(),
            Some("https://example.com/bob.png")
        );
    }

    #[test]
    fn world_ids_are_replaced_and_world_picture_is_attached() {
        let mut config = active_config();
        config.rules.get_mut("friend-online").unwrap().body = "Dans {location}".into();
        let mut overlay = ScreenOverlay::from_config(config, PathBuf::from("test.json"));
        let friends = HashMap::new();
        let favorites = HashSet::new();
        let worlds = HashMap::new();
        overlay.ingest(
            [&online(1, "usr_a")],
            &friends,
            &favorites,
            &worlds,
            &worlds,
        );
        let toasts = overlay.ingest(
            [&online(2, "usr_a")],
            &friends,
            &favorites,
            &worlds,
            &worlds,
        );
        assert_eq!(toasts[0].body, "Dans Monde VRChat");

        overlay.resolve_world(
            "wrld_test",
            "Le monde de Bob",
            Some("https://example.com/world.png"),
        );
        assert_eq!(overlay.toasts[0].body, "Dans Le monde de Bob");
        assert_eq!(
            overlay.toasts[0].world_image_url.as_deref(),
            Some("https://example.com/world.png")
        );
    }
}
