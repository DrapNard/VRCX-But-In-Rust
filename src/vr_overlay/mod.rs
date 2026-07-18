//! Optional VR integration. This entire module is removed when `vr-overlay` is disabled.

pub mod config;
pub mod notification;
#[cfg(feature = "vr-wrist")]
pub mod wrist;

pub use config::OverlayConfig;
#[allow(unused_imports)]
pub use notification::{Notification, NotificationError, NotificationRouter};

use std::{fs, path::Path};

/// Loads JSON configuration. Unknown fields are rejected to catch misspelled options.
pub fn load_config(path: impl AsRef<Path>) -> Result<OverlayConfig, ConfigError> {
    let value = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&value)?)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read overlay configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid overlay configuration: {0}")]
    Json(#[from] serde_json::Error),
}
