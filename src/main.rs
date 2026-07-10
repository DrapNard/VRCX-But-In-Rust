#![allow(dead_code)]

mod api;
mod backend;
mod client;
mod error;
mod models;
mod session;
mod store;
mod ui;
mod websocket;

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
