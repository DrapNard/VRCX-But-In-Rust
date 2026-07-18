use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OverlayConfig {
    pub enabled: bool,
    pub notifications: NotificationConfig,
    #[cfg(feature = "vr-wrist")]
    pub wrist: WristConfig,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notifications: NotificationConfig::default(),
            #[cfg(feature = "vr-wrist")]
            wrist: WristConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationConfig {
    pub targets: Vec<NotificationTarget>,
    pub timeout_seconds: f32,
    pub height: u16,
    pub opacity: f32,
    pub volume: f32,
    pub sound: String,
    pub icon: String,
    pub source_name: String,
    /// Empty means all categories. Examples: `invite`, `friend-online`, `group`.
    pub categories: BTreeSet<String>,
    pub muted_categories: BTreeSet<String>,
    pub include_title: bool,
    pub include_body: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            targets: vec![NotificationTarget::Custom],
            timeout_seconds: 5.0,
            height: 175,
            opacity: 0.95,
            volume: 0.65,
            sound: "default".into(),
            icon: "default".into(),
            source_name: "VRCX-BIR".into(),
            categories: BTreeSet::new(),
            muted_categories: BTreeSet::new(),
            include_title: true,
            include_body: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationTarget {
    XsOverlay,
    OvrToolkit,
    Custom,
}

#[cfg(feature = "vr-wrist")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WristConfig {
    pub runtime: WristRuntime,
    pub hand: Hand,
    pub width_metres: f32,
    pub resolution: [u32; 2],
    pub offset_metres: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub scale: f32,
    pub opacity: f32,
    pub visible_angle_degrees: f32,
    pub refresh_hz: u16,
    pub theme: WristTheme,
    pub sections: Vec<WristSection>,
    pub custom_endpoint: SocketAddr,
}

#[cfg(feature = "vr-wrist")]
impl Default for WristConfig {
    fn default() -> Self {
        Self {
            runtime: WristRuntime::Auto,
            hand: Hand::Left,
            width_metres: 0.16,
            resolution: [720, 720],
            offset_metres: [0.0, 0.035, 0.02],
            rotation_degrees: [-35.0, 0.0, 0.0],
            scale: 1.0,
            opacity: 0.92,
            visible_angle_degrees: 55.0,
            refresh_hz: 10,
            theme: WristTheme::default(),
            sections: vec![
                WristSection::Clock,
                WristSection::CurrentInstance,
                WristSection::OnlineFriends,
                WristSection::RecentNotifications,
                WristSection::Connection,
            ],
            custom_endpoint: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 29472),
        }
    }
}

#[cfg(feature = "vr-wrist")]
impl WristConfig {
    pub fn frame_interval(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.refresh_hz.max(1) as f64)
    }
}

#[cfg(feature = "vr-wrist")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WristRuntime {
    Auto,
    SteamVr,
    WayVr,
}
#[cfg(feature = "vr-wrist")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Hand {
    Left,
    Right,
}
#[cfg(feature = "vr-wrist")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WristSection {
    Clock,
    CurrentInstance,
    OnlineFriends,
    RecentNotifications,
    Connection,
    Battery,
    Custom,
}

#[cfg(feature = "vr-wrist")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WristTheme {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub danger: String,
    pub font_size: u16,
    pub corner_radius: u16,
    pub padding: u16,
}
#[cfg(feature = "vr-wrist")]
impl Default for WristTheme {
    fn default() -> Self {
        Self {
            background: "#11131EEB".into(),
            foreground: "#F5F7FF".into(),
            accent: "#6C8CFF".into(),
            danger: "#FF6174".into(),
            font_size: 24,
            corner_radius: 22,
            padding: 20,
        }
    }
}
