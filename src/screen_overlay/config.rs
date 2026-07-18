use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Runtime configuration for the lightweight desktop notification overlay.
///
/// Rules are keyed by pipeline event names such as `friend-online`,
/// `friend-offline`, `notification`, or `instance-queue-ready`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScreenOverlayConfig {
    pub enabled: bool,
    pub width: u16,
    pub margin: u16,
    pub spacing: u16,
    pub max_visible: usize,
    pub default_duration_seconds: f32,
    pub default_opacity: f32,
    pub background: String,
    pub foreground: String,
    pub rules: BTreeMap<String, EventRule>,
}

impl Default for ScreenOverlayConfig {
    fn default() -> Self {
        let mut rules = BTreeMap::new();
        rules.insert(
            "friend-online".into(),
            EventRule {
                enabled: true,
                title: "{name} est en ligne".into(),
                body: "Connecte depuis {platform}".into(),
                accent: "#6C8CFF".into(),
                ..EventRule::default()
            },
        );
        rules.insert(
            "notification".into(),
            EventRule {
                enabled: true,
                title: "Notification VRChat".into(),
                body: "{message}".into(),
                accent: "#9B7BFF".into(),
                ..EventRule::default()
            },
        );
        rules.insert(
            "instance-queue-ready".into(),
            EventRule {
                enabled: true,
                title: "Instance disponible".into(),
                body: "La file d'attente est terminee.".into(),
                accent: "#54D39B".into(),
                ..EventRule::default()
            },
        );
        for event in [
            "friend-offline",
            "friend-active",
            "friend-location",
            "friend-add",
            "friend-delete",
            "instance-queue-joined",
            "user-location",
        ] {
            rules.insert(event.into(), EventRule::default());
        }

        Self {
            enabled: false,
            width: 380,
            margin: 18,
            spacing: 8,
            max_visible: 4,
            default_duration_seconds: 5.0,
            default_opacity: 0.86,
            background: "#11131E".into(),
            foreground: "#F5F7FF".into(),
            rules,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EventRule {
    pub enabled: bool,
    /// Explicit selection. When empty, `favorites_only` chooses favorites or everyone.
    pub selected_users: BTreeSet<String>,
    /// Used when `selected_users` is empty. Non-user events ignore this filter.
    pub favorites_only: bool,
    pub title: String,
    pub body: String,
    pub accent: String,
    pub show_profile_picture: bool,
    pub show_world_picture: bool,
    /// `None` uses `default_duration_seconds`.
    pub duration_seconds: Option<f32>,
    /// `None` uses `default_opacity`.
    pub opacity: Option<f32>,
}

impl Default for EventRule {
    fn default() -> Self {
        Self {
            enabled: false,
            selected_users: BTreeSet::new(),
            favorites_only: true,
            title: "{event}".into(),
            body: "{message}".into(),
            accent: "#6C8CFF".into(),
            show_profile_picture: true,
            show_world_picture: true,
            duration_seconds: None,
            opacity: None,
        }
    }
}

impl ScreenOverlayConfig {
    pub fn sanitized(mut self) -> Self {
        for (event, rule) in ScreenOverlayConfig::default().rules {
            self.rules.entry(event).or_insert(rule);
        }
        self.width = self.width.clamp(240, 720);
        self.margin = self.margin.min(128);
        self.spacing = self.spacing.min(48);
        self.max_visible = self.max_visible.clamp(1, 8);
        self.default_duration_seconds = self.default_duration_seconds.clamp(1.0, 30.0);
        self.default_opacity = self.default_opacity.clamp(0.15, 1.0);
        for rule in self.rules.values_mut() {
            rule.duration_seconds = rule.duration_seconds.map(|v| v.clamp(1.0, 30.0));
            rule.opacity = rule.opacity.map(|v| v.clamp(0.15, 1.0));
        }
        self
    }
}
