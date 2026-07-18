//! Lightweight, event-driven on-screen notifications.
//!
//! No window and no timer exist while the overlay is idle. A small transparent,
//! click-through window is created only while one or more toasts are visible.

mod config;

pub use config::ScreenOverlayConfig;

use crate::{store::RecentPipelineEvent, websocket::PipelineEvent};
use iced::{
    Border, Color, Element, Fill, Length, Point, Size,
    widget::{column, container, row, rule, text},
    window,
};
use serde_json::Value;
use std::{collections::VecDeque, fs, path::PathBuf, time::Duration};

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
}

#[derive(Debug)]
pub struct ScreenOverlay {
    pub config: ScreenOverlayConfig,
    pub config_path: PathBuf,
    pub window_id: Option<window::Id>,
    toasts: VecDeque<Toast>,
    last_sequence: Option<u64>,
    next_toast_id: u64,
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
        }
    }

    /// Returns newly accepted toasts. The first snapshot only establishes a watermark,
    /// so cached events never produce a burst when the application starts.
    pub fn ingest<'a>(
        &'a mut self,
        events: impl IntoIterator<Item = &'a RecentPipelineEvent>,
    ) -> Vec<Toast> {
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
                if let Some(toast) = self.toast_for(recent) {
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

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        let content = self.toasts.iter().fold(
            column![].spacing(self.config.spacing as f32),
            |content, toast| {
                let accent = toast.accent;
                let background = Color {
                    a: toast.opacity,
                    ..toast.background
                };
                let foreground = toast.foreground;
                let card = row![
                    rule::vertical(4).style(move |_| rule::Style {
                        color: accent,
                        radius: Border::default().radius,
                        fill_mode: rule::FillMode::Full,
                        snap: true,
                    }),
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

    fn toast_for(&mut self, recent: &RecentPipelineEvent) -> Option<Toast> {
        let fields = event_fields(&recent.event);
        let rule = self.config.rules.get(fields.key)?.clone();
        if !rule.enabled
            || (!rule.selected_users.is_empty()
                && fields
                    .user_id
                    .is_none_or(|id| !rule.selected_users.contains(id)))
        {
            return None;
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
        })
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
    location: &'a str,
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
            location: &content.location,
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendOffline(content) => EventFields {
            key: "friend-offline",
            event: "Friend offline",
            name: content.user_id.clone(),
            user_id: Some(&content.user_id),
            platform: &content.platform,
            location: empty,
            message: content.user_id.clone(),
        },
        PipelineEvent::FriendActive(content) => EventFields {
            key: "friend-active",
            event: "Friend active",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: &content.platform,
            location: empty,
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendLocation(content) => EventFields {
            key: "friend-location",
            event: "Friend location",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: empty,
            location: &content.location,
            message: content.location.clone(),
        },
        PipelineEvent::FriendAdd(content) => EventFields {
            key: "friend-add",
            event: "Friend added",
            name: display_name(&content.user, &content.user_id),
            user_id: Some(&content.user_id),
            platform: empty,
            location: empty,
            message: display_name(&content.user, &content.user_id),
        },
        PipelineEvent::FriendDelete { user_id } => EventFields {
            key: "friend-delete",
            event: "Friend removed",
            name: user_id.clone(),
            user_id: Some(user_id),
            platform: empty,
            location: empty,
            message: user_id.clone(),
        },
        PipelineEvent::Notification(value) | PipelineEvent::NotificationV2(value) => EventFields {
            key: "notification",
            event: "Notification",
            name: json_string(value, "senderUsername"),
            user_id: value.get("senderUserId").and_then(Value::as_str),
            platform: empty,
            location: empty,
            message: json_string(value, "message"),
        },
        PipelineEvent::InstanceQueueReady(content) => EventFields {
            key: "instance-queue-ready",
            event: "Instance ready",
            name: String::new(),
            user_id: None,
            platform: empty,
            location: &content.instance_location,
            message: content.instance_location.clone(),
        },
        PipelineEvent::InstanceQueueJoined(content) => EventFields {
            key: "instance-queue-joined",
            event: "Instance queue",
            name: String::new(),
            user_id: None,
            platform: empty,
            location: &content.instance_location,
            message: format!("Position {}", content.position),
        },
        other => EventFields {
            key: event_key(other),
            event: event_key(other),
            name: String::new(),
            user_id: None,
            platform: empty,
            location: empty,
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

fn render_template(template: &str, fields: &EventFields<'_>) -> String {
    template
        .replace("{event}", fields.event)
        .replace("{name}", &fields.name)
        .replace("{user_id}", fields.user_id.unwrap_or_default())
        .replace("{platform}", fields.platform)
        .replace("{location}", fields.location)
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
    use crate::websocket::event::FriendOnlineContent;

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
        let mut overlay =
            ScreenOverlay::from_config(ScreenOverlayConfig::default(), PathBuf::from("test.json"));
        assert!(overlay.ingest([&online(1, "usr_a")]).is_empty());
        assert_eq!(
            overlay
                .ingest([&online(1, "usr_a"), &online(2, "usr_a")])
                .len(),
            1
        );
        assert!(overlay.ingest([&online(2, "usr_a")]).is_empty());
    }

    #[test]
    fn selected_user_filter_is_applied() {
        let mut overlay =
            ScreenOverlay::from_config(ScreenOverlayConfig::default(), PathBuf::from("test.json"));
        overlay
            .config
            .rules
            .get_mut("friend-online")
            .unwrap()
            .selected_users = ["usr_wanted".to_string()].into_iter().collect();
        overlay.ingest([&online(1, "usr_wanted")]);
        assert!(overlay.ingest([&online(2, "usr_other")]).is_empty());
        assert_eq!(overlay.ingest([&online(3, "usr_wanted")]).len(), 1);
    }

    #[test]
    fn preview_works_even_when_live_overlay_is_disabled() {
        let mut config = ScreenOverlayConfig::default();
        config.enabled = false;
        let mut overlay = ScreenOverlay::from_config(config, PathBuf::from("test.json"));
        let toast = overlay.preview("friend-online");
        assert!(overlay.has_toasts());
        assert!(toast.title.contains("Utilisateur test"));
        assert!(overlay.expire(toast.id));
    }
}
