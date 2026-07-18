#![allow(dead_code)]

mod api;
mod backend;
mod client;
mod error;
mod models;
mod session;
mod store;
mod ui;
// VR is intentionally absent from every macOS build, even when a VR feature is requested.
#[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
mod vr_overlay;
mod websocket;

rust_i18n::i18n!("locales");

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vrcx_but_in_rust=info".into()),
        )
        .with_target(false)
        .compact()
        .init();

    tracing::info!("starting VRCX Rust");
    ui::run()
}
