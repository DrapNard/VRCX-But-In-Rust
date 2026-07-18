# VRCX-But-In-Rust (or VRCX-BIR)

---

A native, lightweight, cross-platform VRChat companion app written in Rust.

This project is inspired by the idea of tools like VRCX, but rebuilt with a native Rust stack instead of a web/Electron stack. The goal is to provide a fast, low-memory, pleasant desktop app for managing and exploring VRChat data such as users, friends, worlds, instances, avatars, favorites, groups, notifications, and account-related information.

This project is unofficial and is not affiliated with VRChat Inc.

Goals

* Native desktop application
* Lightweight memory usage
* Cross-platform support
* Clean Rust architecture
* Strongly typed VRChat API models
* Local cache/database support
* Good offline behavior where possible
* Nice and responsive UI
* No embedded browser or Electron runtime
* Modular design for future features

Tech Stack

The current intended stack is:

* Rust for the application logic
* iced for the native GUI
* reqwest or equivalent HTTP client for API calls
* tokio for async runtime
* serde for JSON serialization/deserialization
* SQLite / lightweight embedded DB for cache and local state
* tungstenite / tokio-tungstenite or equivalent for WebSocket support
* tracing for logging

The exact dependencies may evolve as the project matures.

> [!WARNING]
> Project Status
>
>This project is currently experimental and under active development.
>
>Many models, API routes, cache systems, and UI screens are still being designed. Breaking changes should be expected until the project reaches a stable beta.

Planned Features

Account

* Login/session support
* Current user profile
* Account status
* Two-factor authentication flow
* Token/session persistence

Friends

* Friend list
* Online/offline status
* Current location
* Favorite friends
* Friend notes
* Friend requests

Users

* User profile viewer
* Trust rank parsing
* Tags parsing
* Avatar/world permissions
* Languages
* Supporter/admin/moderation-related flags where available

Worlds

* World search
* World details
* Tags and content warnings
* Capacity, visits, popularity, heat
* Favorites
* Labs/community/system flags

Instances

* Instance details
* Instance type
* Region
* Occupants
* Queue information
* Access restrictions
* World-instance parsing

Avatars

* Current avatar
* Fallback avatar
* Avatar metadata
* Favorites
* Permissions

Cache

* Local cache for API responses
* Offline-friendly data display
* Cache invalidation
* Optional lightweight database backend

UI

* Native desktop layout
* Search panels
* Profile/world/avatar cards
* Fast navigation
* Theme support
* Low RAM usage

Non-Goals

This project does not aim to:

* Clone VRCX 1:1
* Embed a browser UI
* Use Electron, Tauri, or a web frontend by default
* Bypass VRChat API rules
* Automate abusive behavior
* Provide moderation evasion tools
* Scrape private data
* Replace the official VRChat client

Building

Install Rust:
```shell
rustup update
```
Clone the repository:
```shell
git clone <repo-url>
cd vrcx-but-in-rust
```
Build the project:
```shell
cargo build
```
Run the project:
```shell
cargo run
```
For release builds:
```shell
cargo build --release
```

Optional VR module (nothing is included in the default build):
```shell
# XSOverlay notifications only
cargo build --release --features vr-notifications-xs

# Wrist overlay model for a SteamVR renderer
cargo build --release --features vr-wrist-steamvr

# Wrist overlay model for a WayVR renderer
cargo build --release --features vr-wrist-wayvr

# Complete Windows build (XSOverlay + OVR Toolkit + SteamVR/OpenVR)
cargo build --release --features vr-windows

# Complete Linux build (Windows backends + WayVR/OpenXR)
cargo build --release --features vr-linux
```

The VR module is configured with JSON through `vr_overlay::load_config`. Notification
categories, appearance, audio, opacity and targets are configurable. The wrist model exposes
the essential clock, instance, friends, notifications and connection sections by default, plus
custom data and a replaceable renderer for runtime-specific integrations.

`vr-wrist-steamvr` connects to `IVROverlay`, anchors the surface to the configured controller
and submits RGBA8 frames to SteamVR's compositor. It is available on Windows and Linux.
`vr-wrist-wayvr` dynamically probes the active OpenXR loader for `XR_EXTX_overlay`. When that
provisional extension is absent, the application reports WayVR as the required compositor bridge;
a standard OpenXR quad layer cannot overlay a different application's session.

Platform matrix: macOS builds contain no VR module or VR runtime dependency, regardless of the
requested feature flags. Windows builds contain the OpenVR/SteamVR path but exclude WayVR and
OpenXR. Linux builds may contain both paths.
Development Commands

Format code:
```shell
cargo fmt
```
Run Clippy:
```shell
cargo clippy --all-targets --all-features
```
Run tests:
```shell
cargo test
```
Check without building binaries:
```shell
cargo check
```
Recommended full local check:
```shell
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```
API Model Design

The project tries to keep VRChat API models strongly typed while preserving unknown or future fields when useful.

For example, tags should avoid becoming a loose Vec<String> everywhere. Instead, known tags should be parsed into enums or structured fields, while unknown raw tags should be preserved.

Example direction:
```rust
pub struct Tags {
pub trust_rank: TrustRank,
pub troll_state: TrollState,
pub supporter_state: SupporterState,
pub admin_tags: AdminTags,
pub permissions: Permissions,
pub languages: Vec<LanguageTag>,
pub raw: Vec<String>,
}
```
For world tags, known content warnings and system flags should be parsed into explicit enums.
```rust
pub enum ContentWarning {
Adult,
Combat,
Featured,
Gore,
Horror,
Other,
Sex,
Violence,
}
pub enum SystemFlag {
Approved,
CreatedRecently,
Labs,
UpdatedRecently,
PublishedRecently,
MonetizedWorld,
PositiveFunToExplore,
JamTags,
}
```
Unknown tags should not be discarded. They should remain accessible through a raw/unknown field.

Error Handling

The project should avoid panics in normal application flow.

Prefer:

* typed error enums
* thiserror
* anyhow only at app boundaries or prototype-level code
* clear API error variants
* recoverable UI states

Bad:
```rust
let value = something.unwrap();
```
Better:
```rust
let value = something.ok_or(AppError::MissingField("value"))?;
```
Logging

Use structured logging through tracing.

Avoid random println! debugging in committed code unless it is part of CLI output or temporary prototype code.

Recommended:
```rust
tracing::info!("loaded user profile");
tracing::warn!(user_id = %id, "failed to refresh user");
tracing::error!(error = ?err, "api request failed");
```
Configuration

Configuration should eventually support:

* session storage
* cache path
* database path
* theme settings
* API behavior
* privacy options
* debug logging

Sensitive information must not be committed.

Privacy

This app may handle sensitive account/session data.

Contributors must be careful with:

* auth cookies
* tokens
* user IDs
* private locations
* friend lists
* cached API responses
* debug logs

Never commit real session data, API responses containing private data, or screenshots containing private account information.

VRChat API Usage

This project should respect VRChat API behavior and limits.

Do not add features intended to spam requests, bypass restrictions, evade moderation, scrape private data, or abuse the platform.

Caching, rate limiting, and careful request behavior are important project goals.

License

* MIT

Disclaimer

This project is unofficial and community-made. VRChat is a trademark of VRChat Inc. This project is not endorsed by or affiliated with VRChat Inc.
