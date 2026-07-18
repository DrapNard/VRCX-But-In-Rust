use super::config::{NotificationConfig, NotificationTarget};
use serde::{Deserialize, Serialize};
use std::{future::Future, net::UdpSocket, pin::Pin, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub category: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

pub trait CustomNotificationSink: Send + Sync {
    fn send<'a>(
        &'a self,
        notification: &'a Notification,
    ) -> Pin<Box<dyn Future<Output = Result<(), NotificationError>> + Send + 'a>>;
}

pub struct NotificationRouter {
    config: NotificationConfig,
    custom: Option<Arc<dyn CustomNotificationSink>>,
}

impl NotificationRouter {
    pub fn new(config: NotificationConfig) -> Self {
        Self {
            config,
            custom: None,
        }
    }
    pub fn with_custom_sink(mut self, sink: Arc<dyn CustomNotificationSink>) -> Self {
        self.custom = Some(sink);
        self
    }
    pub fn accepts(&self, category: &str) -> bool {
        !self.config.muted_categories.contains(category)
            && (self.config.categories.is_empty() || self.config.categories.contains(category))
    }
    pub async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        if !self.accepts(&notification.category) {
            return Ok(());
        }
        for target in &self.config.targets {
            match target {
                NotificationTarget::XsOverlay => self.send_xs(notification)?,
                NotificationTarget::OvrToolkit => self.send_ovr_toolkit(notification).await?,
                NotificationTarget::Custom => {
                    if let Some(sink) = &self.custom {
                        sink.send(notification).await?
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "vr-notifications-xs")]
    fn send_xs(&self, n: &Notification) -> Result<(), NotificationError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Packet<'a> {
            message_type: u8,
            index: u8,
            timeout: f32,
            height: u16,
            opacity: f32,
            volume: f32,
            audio_path: &'a str,
            title: &'a str,
            content: &'a str,
            source_app: &'a str,
            use_base64_icon: bool,
            icon: &'a str,
        }
        let title = if self.config.include_title {
            &n.title
        } else {
            ""
        };
        let body = if self.config.include_body {
            &n.body
        } else {
            ""
        };
        let icon = n.icon.as_deref().unwrap_or(&self.config.icon);
        let packet = Packet {
            message_type: 1,
            index: 0,
            timeout: self.config.timeout_seconds,
            height: self.config.height,
            opacity: self.config.opacity.clamp(0.0, 1.0),
            volume: self.config.volume.clamp(0.0, 1.0),
            audio_path: &self.config.sound,
            title,
            content: body,
            source_app: &self.config.source_name,
            use_base64_icon: false,
            icon,
        };
        let bytes = serde_json::to_vec(&packet)?;
        UdpSocket::bind("127.0.0.1:0")?.send_to(&bytes, "127.0.0.1:42069")?;
        Ok(())
    }
    #[cfg(not(feature = "vr-notifications-xs"))]
    fn send_xs(&self, _: &Notification) -> Result<(), NotificationError> {
        Err(NotificationError::BackendNotCompiled("vr-notifications-xs"))
    }

    // OVR Toolkit's API has changed across releases; integrations provide a custom sink until
    // an explicitly versioned protocol is selected, instead of silently sending invalid data.
    async fn send_ovr_toolkit(&self, n: &Notification) -> Result<(), NotificationError> {
        #[cfg(feature = "vr-notifications-ovr-toolkit")]
        if let Some(sink) = &self.custom {
            return sink.send(n).await;
        }
        Err(NotificationError::BackendNotCompiled(
            "versioned OVR Toolkit custom sink",
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("notification backend unavailable: {0}")]
    BackendNotCompiled(&'static str),
    #[error("notification I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("notification serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("custom notification backend failed: {0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn category_filters_are_applied() {
        let mut c = NotificationConfig::default();
        c.categories = BTreeSet::from(["invite".into()]);
        let r = NotificationRouter::new(c);
        assert!(r.accepts("invite"));
        assert!(!r.accepts("friend-online"));
    }
}
