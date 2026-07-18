use super::config::WristConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(all(
    feature = "vr-wrist-steamvr",
    any(target_os = "windows", target_os = "linux")
))]
mod openvr_runtime;
// WayVR is Linux-only; Windows builds never compile its OpenXR path.
#[cfg(all(feature = "vr-wrist-wayvr", target_os = "linux"))]
mod openxr_runtime;

#[cfg(all(
    feature = "vr-wrist-steamvr",
    any(target_os = "windows", target_os = "linux")
))]
pub use openvr_runtime::OpenVrWristRenderer;
#[cfg(all(feature = "vr-wrist-wayvr", target_os = "linux"))]
#[allow(unused_imports)]
pub use openxr_runtime::{OpenXrCapability, probe_openxr_overlay};

/// Runtime-neutral state rendered by either an OpenVR overlay or a WayVR surface.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WristSnapshot {
    pub clock: String,
    pub current_world: Option<String>,
    pub current_instance: Option<String>,
    pub online_friends: Vec<WristFriend>,
    pub notifications: Vec<WristNotification>,
    pub connected: bool,
    pub battery_percent: Option<u8>,
    pub custom: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WristFriend {
    pub display_name: String,
    pub status: String,
    pub location: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WristNotification {
    pub title: String,
    pub body: String,
    pub timestamp: String,
    pub category: String,
}

pub trait WristRenderer: Send {
    fn configure(&mut self, config: &WristConfig) -> Result<(), WristError>;
    fn render(&mut self, snapshot: &WristSnapshot) -> Result<(), WristError>;
    fn set_visible(&mut self, visible: bool) -> Result<(), WristError>;
}

/// Pixel buffer sent to the native VR compositor. Pixels use straight-alpha RGBA8.
#[derive(Debug, Clone)]
pub struct WristFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl WristFrame {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, WristError> {
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err(WristError::Render(format!(
                "invalid RGBA frame: {} bytes, expected {expected}",
                rgba.len()
            )));
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }
}

/// Implemented by renderers that accept an already rasterized GPU/compositor frame.
pub trait WristFrameSink: Send {
    fn submit_frame(&mut self, frame: &WristFrame) -> Result<(), WristError>;
    fn set_visible(&mut self, visible: bool) -> Result<(), WristError>;
}
#[derive(Debug, thiserror::Error)]
pub enum WristError {
    #[error("VR runtime unavailable: {0}")]
    Runtime(String),
    #[error("wrist renderer failed: {0}")]
    Render(String),
}
