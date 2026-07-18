use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};

use futures_util::{StreamExt, stream};
use iced::{
    Border, Color, ContentFit, Element, Fill, Length, Subscription, Task, Theme, mouse,
    widget::{
        Space, button, column, container, image, mouse_area, opaque, row, rule, scrollable, stack,
        text, text_input,
    },
    window,
};
use rust_i18n::t;
use serde_json::Value;

const RESOURCE_PAGE_LIMIT: u32 = 50;
const THUMBNAIL_MAX_EDGE: u32 = 96;
const MAX_FRIEND_THUMBNAILS: usize = 90;
const MAX_RELATION_THUMBNAILS: usize = 80;
const INITIAL_RESOURCE_ROWS: usize = 28;
const RESOURCE_ROW_INCREMENT: usize = 28;
const INITIAL_FRIEND_SECTION_ROWS: usize = 18;
const FRIEND_SECTION_ROW_INCREMENT: usize = 18;
const INITIAL_RELATION_ROWS: usize = 32;
const RELATION_ROW_INCREMENT: usize = 32;
const INFINITE_SCROLL_THRESHOLD: f32 = 0.86;
const SCROLLBAR_SPACING: f32 = 8.0;

use crate::{
    api::{
        PaginationQuery, avatars::AvatarSearchQuery, favorites::FavoritesQuery,
        groups::GroupSearchQuery, inventory::InventoryQuery, users::UserSearchQuery,
        worlds::WorldSearchQuery,
    },
    backend::{AuthenticatedSession, Backend, BackendConfig, LoginOutcome, SessionAccount},
    models::{inventory::InventoryConsume, invite::InviteMessageType},
    session::auth::TwoFactorMethod,
    store::{AppSnapshot, WebSocketStatus},
    websocket::PipelineEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Page {
    Dashboard,
    Friends,
    Notifications,
    Worlds,
    Users,
    Groups,
    Avatars,
    MyAvatars,
    Inventory,
    Calendar,
    Economy,
    Files,
    Jams,
    Props,
    Favorites,
    Instances,
    Invites,
    Moderation,
    Prints,
    System,
    Api,
    #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
    VrOverlay,
}

impl Page {
    const ALL: &'static [Page] = &[
        Page::Dashboard,
        Page::Notifications,
        Page::Worlds,
        Page::Groups,
        Page::MyAvatars,
        Page::Avatars,
        Page::Inventory,
        Page::Calendar,
        Page::Economy,
        Page::Files,
        Page::Jams,
        Page::Props,
        Page::Favorites,
        Page::Instances,
        Page::Invites,
        Page::Moderation,
        Page::Prints,
        Page::System,
        Page::Api,
        #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
        Page::VrOverlay,
    ];

    const ESSENTIAL: &'static [Page] = &[
        Page::Dashboard,
        Page::Notifications,
        Page::Worlds,
        Page::Groups,
        Page::MyAvatars,
        Page::Favorites,
        #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
        Page::VrOverlay,
    ];

    const SEARCH_FILTERS: [Page; 7] = [
        Page::Users,
        Page::Worlds,
        Page::Groups,
        Page::Avatars,
        Page::MyAvatars,
        Page::Favorites,
        Page::Prints,
    ];

    fn label(self) -> String {
        match self {
            Self::Dashboard => t!("pages.dashboard").to_string(),
            Self::Friends => t!("pages.friends").to_string(),
            Self::Notifications => t!("pages.notifications").to_string(),
            Self::Worlds => t!("pages.worlds").to_string(),
            Self::Users => t!("pages.users").to_string(),
            Self::Groups => t!("pages.groups").to_string(),
            Self::Avatars => t!("pages.avatars").to_string(),
            Self::MyAvatars => t!("pages.my_avatars").to_string(),
            Self::Inventory => t!("pages.inventory").to_string(),
            Self::Calendar => t!("pages.calendar").to_string(),
            Self::Economy => t!("pages.economy").to_string(),
            Self::Files => t!("pages.files").to_string(),
            Self::Jams => t!("pages.jams").to_string(),
            Self::Props => t!("pages.props").to_string(),
            Self::Favorites => t!("pages.favorites").to_string(),
            Self::Instances => t!("pages.instances").to_string(),
            Self::Invites => t!("pages.invites").to_string(),
            Self::Moderation => t!("pages.moderation").to_string(),
            Self::Prints => t!("pages.prints").to_string(),
            Self::System => t!("pages.system").to_string(),
            Self::Api => t!("pages.api").to_string(),
            #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
            Self::VrOverlay => t!("pages.vr_overlay").to_string(),
        }
    }

    fn is_searchable(self) -> bool {
        matches!(
            self,
            Self::Users
                | Self::Worlds
                | Self::Groups
                | Self::Avatars
                | Self::MyAvatars
                | Self::Favorites
                | Self::Prints
        )
    }
}

#[derive(Debug, Clone)]
struct ResultItem {
    id: String,
    title: String,
    subtitle: String,
    thumbnail_url: Option<String>,
    round_thumbnail: bool,
    platforms: Vec<ContentPlatform>,
    badges: Vec<String>,
    trust_rank: Option<String>,
    favorite_id: Option<String>,
    raw: Value,
}

#[derive(Debug, Clone)]
struct FavoriteFriendGroup {
    name: String,
    display_name: String,
    friend_ids: HashSet<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OwnAvatarQuery {
    #[serde(flatten)]
    page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<String>,
    user: &'static str,
    release_status: &'static str,
    order: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentPlatform {
    Windows,
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AvatarPerformance {
    Excellent,
    Good,
    Medium,
    Poor,
    VeryPoor,
}

#[derive(Debug, Clone)]
struct NavCategory {
    name: String,
    pages: Vec<Page>,
}

impl NavCategory {
    fn essentials() -> Self {
        Self {
            name: t!("nav.essentials").to_string(),
            pages: Page::ESSENTIAL.to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Locale {
    English,
    French,
}

impl Locale {
    const ALL: [Self; 2] = [Self::English, Self::French];

    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::English => "EN",
            Self::French => "FR",
        }
    }

    fn from_environment() -> Self {
        std::env::var("LANG")
            .ok()
            .filter(|lang| lang.to_lowercase().starts_with("fr"))
            .map(|_| Self::French)
            .unwrap_or(Self::English)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailTab {
    Overview,
    MutualFriends,
    MutualGroups,
    Groups,
    FriendGroups,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum FriendSection {
    Favorites,
    InVrchat,
    Private,
    Web,
    Offline,
}

impl FriendSection {
    const ALL: [Self; 5] = [
        Self::Favorites,
        Self::InVrchat,
        Self::Private,
        Self::Web,
        Self::Offline,
    ];

    fn label(self) -> String {
        match self {
            Self::InVrchat => t!("friends.sections.in_vrchat").to_string(),
            Self::Private => t!("friends.sections.private").to_string(),
            Self::Web => t!("friends.sections.web").to_string(),
            Self::Favorites => t!("friends.sections.favorites").to_string(),
            Self::Offline => t!("friends.sections.offline").to_string(),
        }
    }
}

fn default_friend_section_row_limits() -> HashMap<FriendSection, usize> {
    FriendSection::ALL
        .into_iter()
        .map(|section| (section, INITIAL_FRIEND_SECTION_ROWS))
        .collect()
}

#[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
fn vr_build_summary() -> String {
    let mut backends = Vec::new();
    if cfg!(feature = "vr-notifications-xs") {
        backends.push("XSOverlay notifications");
    }
    if cfg!(feature = "vr-notifications-ovr-toolkit") {
        backends.push("OVR Toolkit notifications");
    }
    if cfg!(all(
        feature = "vr-wrist-steamvr",
        any(target_os = "windows", target_os = "linux")
    )) {
        backends.push("SteamVR / OpenVR wrist overlay");
    }
    if cfg!(all(feature = "vr-wrist-wayvr", target_os = "linux")) {
        backends.push("WayVR / OpenXR wrist overlay");
    }
    if backends.is_empty() {
        "VR core only (no notification or wrist backend)".to_string()
    } else {
        backends.join("\n")
    }
}

#[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
fn vr_build_command() -> &'static str {
    if cfg!(target_os = "windows") {
        "cargo build --release --features vr-windows"
    } else {
        "cargo build --release --features vr-linux"
    }
}

impl DetailTab {
    const ALL: [Self; 5] = [
        Self::Overview,
        Self::MutualFriends,
        Self::MutualGroups,
        Self::Groups,
        Self::FriendGroups,
    ];

    fn label(self) -> String {
        match self {
            Self::Overview => t!("detail.tabs.overview").to_string(),
            Self::MutualFriends => t!("detail.tabs.mutual_friends").to_string(),
            Self::MutualGroups => t!("detail.tabs.mutual_groups").to_string(),
            Self::Groups => t!("detail.tabs.groups").to_string(),
            Self::FriendGroups => t!("detail.tabs.friend_groups").to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct UserRelations {
    mutual_friends: Vec<ResultItem>,
    mutual_groups: Vec<ResultItem>,
    groups: Vec<ResultItem>,
    friend_groups: Vec<ResultItem>,
}

#[derive(Clone)]
struct SnapshotSubscription {
    backend_id: usize,
    receiver: tokio::sync::watch::Receiver<AppSnapshot>,
}

impl Hash for SnapshotSubscription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.backend_id.hash(state);
    }
}

pub struct App {
    main_window: window::Id,
    screen_overlay: crate::screen_overlay::ScreenOverlay,
    backend: Option<Arc<Backend>>,
    snapshot: AppSnapshot,
    session: Option<AuthenticatedSession>,
    page: Page,
    username: String,
    password: String,
    two_factor_code: String,
    two_factor_methods: Vec<TwoFactorMethod>,
    selected_two_factor: Option<TwoFactorMethod>,
    search: String,
    results: Vec<ResultItem>,
    selected_item: Option<ResultItem>,
    selected_page: Page,
    selected_detail: Option<Value>,
    hovered_item: Option<String>,
    search_overlay_open: bool,
    detail_loading: bool,
    detail_tab: DetailTab,
    user_relations: UserRelations,
    relations_loading: bool,
    thumbnails: HashMap<String, image::Handle>,
    pending_thumbnails: HashSet<String>,
    world_names: HashMap<String, String>,
    world_image_urls: HashMap<String, String>,
    pending_world_names: HashSet<String>,
    favorite_friend_ids: HashSet<String>,
    favorite_friend_groups: Vec<FavoriteFriendGroup>,
    collapsed_friend_sections: HashSet<FriendSection>,
    resource_row_limit: usize,
    relation_row_limit: usize,
    friend_section_row_limits: HashMap<FriendSection, usize>,
    nav_categories: Vec<NavCategory>,
    collapsed_nav_categories: HashSet<String>,
    panel_selector_open: bool,
    settings_open: bool,
    nav_edit_mode: bool,
    account_menu_open: bool,
    saved_sessions: Vec<SessionAccount>,
    locale: Locale,
    new_category_name: String,
    loading: bool,
    error: Option<String>,
    notice: Option<String>,
    api_method: String,
    api_path: String,
    api_body: String,
    api_response: String,
    overlay_editor_rule: String,
    overlay_editor_duration: String,
    overlay_editor_opacity: String,
}

#[derive(Debug, Clone)]
enum Message {
    MainWindowOpened,
    WindowClosed(window::Id),
    OverlayWindowOpened(window::Id),
    OverlayExpired(u64),
    ReloadScreenOverlay,
    TestScreenOverlay,
    ToggleScreenOverlay,
    SelectOverlayRule(String),
    ToggleOverlayRule,
    OverlayRuleTitleChanged(String),
    OverlayRuleBodyChanged(String),
    OverlayRuleAccentChanged(String),
    OverlayRuleDurationChanged(String),
    OverlayRuleOpacityChanged(String),
    ToggleOverlayRuleProfilePicture,
    ToggleOverlayRuleWorldPicture,
    OverlayRuleEveryone,
    OverlayRuleFavorites,
    ToggleOverlayRuleUser(String),
    SaveScreenOverlayEditor,
    BootFinished(Result<(Vec<SessionAccount>, Option<AuthenticatedSession>), String>),
    UsernameChanged(String),
    PasswordChanged(String),
    Login,
    LoginFinished(Result<LoginOutcome, String>),
    TwoFactorCodeChanged(String),
    SelectTwoFactor(TwoFactorMethod),
    VerifyTwoFactor,
    Logout,
    LogoutFinished(Result<(), String>),
    Navigate(Page),
    SearchChanged(String),
    SubmitSearch,
    SearchPage(Page),
    CloseSearchOverlay,
    Refresh,
    ResultsLoaded(Result<Vec<Value>, String>),
    FavoriteFriendsLoaded(Result<(HashSet<String>, Vec<FavoriteFriendGroup>), String>),
    ToggleFriendSection(FriendSection),
    ResourceScrolled(f32),
    FriendSidebarScrolled(f32),
    RelationScrolled(f32),
    ToggleNavCategory(String),
    OpenPanelSelector,
    ClosePanelSelector,
    ToggleNavEditMode,
    ToggleAccountMenu,
    CloseAccountMenu,
    OpenSettings,
    CloseSettings,
    SavedSessionsLoaded(Result<Vec<SessionAccount>, String>),
    AddSession,
    AddSessionFinished(Result<(), String>),
    SwitchSession(String),
    SwitchSessionFinished(Result<Option<AuthenticatedSession>, String>),
    LocaleSelected(Locale),
    NewCategoryNameChanged(String),
    CreateNavCategory,
    AddPageToNav(String, Page),
    RemovePageFromNav(String, Page),
    OpenItem(usize),
    OpenFriend(String),
    OpenRelated(ResultItem, Page),
    HoverItem(Option<String>),
    CloseItem,
    DetailLoaded(Result<Value, String>),
    DetailTabSelected(DetailTab),
    UserRelationsLoaded(Result<UserRelations, String>),
    ThumbnailLoaded(String, Result<Vec<u8>, String>),
    WorldNameLoaded(String, Result<(String, Option<String>), String>),
    SnapshotLoaded(AppSnapshot),
    Synchronize,
    SynchronizeFinished(Result<(), String>),
    Unfriend(String),
    LeaveGroup(String),
    RemoveFavorite(String),
    DeleteNotification(String),
    ResourceAction(Page, String),
    ApiMethodChanged(String),
    ApiPathChanged(String),
    ApiBodyChanged(String),
    ExecuteApi,
    ApiExecuted(Result<Value, String>),
    ActionFinished(Result<String, String>),
    ClearFeedback,
}

pub fn run() -> iced::Result {
    iced::daemon(App::boot, App::update, App::view_window)
        .title(app_title)
        .theme(app_theme)
        .style(app_style)
        .subscription(App::subscription)
        .run()
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let (main_window, open_main) = window::open(window::Settings {
            size: iced::Size::new(1180.0, 760.0),
            min_size: Some(iced::Size::new(820.0, 560.0)),
            ..Default::default()
        });
        let locale = Locale::from_environment();
        rust_i18n::set_locale(locale.code());

        let mut app = Self {
            main_window,
            screen_overlay: crate::screen_overlay::ScreenOverlay::load(),
            backend: None,
            snapshot: AppSnapshot::default(),
            session: None,
            page: Page::Dashboard,
            username: String::new(),
            password: String::new(),
            two_factor_code: String::new(),
            two_factor_methods: Vec::new(),
            selected_two_factor: None,
            search: String::new(),
            results: Vec::new(),
            selected_item: None,
            selected_page: Page::Users,
            selected_detail: None,
            hovered_item: None,
            search_overlay_open: false,
            detail_loading: false,
            detail_tab: DetailTab::Overview,
            user_relations: UserRelations::default(),
            relations_loading: false,
            thumbnails: HashMap::new(),
            pending_thumbnails: HashSet::new(),
            world_names: HashMap::new(),
            world_image_urls: HashMap::new(),
            pending_world_names: HashSet::new(),
            favorite_friend_ids: HashSet::new(),
            favorite_friend_groups: Vec::new(),
            collapsed_friend_sections: HashSet::from([FriendSection::Offline]),
            resource_row_limit: INITIAL_RESOURCE_ROWS,
            relation_row_limit: INITIAL_RELATION_ROWS,
            friend_section_row_limits: default_friend_section_row_limits(),
            nav_categories: vec![NavCategory::essentials()],
            collapsed_nav_categories: HashSet::new(),
            panel_selector_open: false,
            settings_open: false,
            nav_edit_mode: false,
            account_menu_open: false,
            saved_sessions: Vec::new(),
            locale,
            new_category_name: String::new(),
            loading: true,
            error: None,
            notice: None,
            api_method: "GET".to_string(),
            api_path: String::new(),
            api_body: "{}".to_string(),
            api_response: String::new(),
            overlay_editor_rule: "friend-online".to_string(),
            overlay_editor_duration: String::new(),
            overlay_editor_opacity: String::new(),
        };

        match BackendConfig::for_app().and_then(Backend::open) {
            Ok(backend) => {
                let backend = Arc::new(backend);
                let task_backend = backend.clone();
                app.backend = Some(backend);
                (
                    app,
                    Task::batch([
                        open_main.map(|_| Message::MainWindowOpened),
                        Task::perform(
                            async move {
                                let sessions = task_backend
                                    .saved_sessions()
                                    .map_err(|error| error.to_string())?;
                                let restored = if sessions.len() == 1 {
                                    task_backend
                                        .restore_session()
                                        .await
                                        .map_err(|error| error.to_string())?
                                } else {
                                    None
                                };
                                Ok((sessions, restored))
                            },
                            Message::BootFinished,
                        ),
                    ]),
                )
            }
            Err(error) => {
                app.loading = false;
                app.error = Some(error.to_string());
                (app, open_main.map(|_| Message::MainWindowOpened))
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainWindowOpened => Task::none(),
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    iced::exit()
                } else {
                    if self.screen_overlay.window_id == Some(id) {
                        self.screen_overlay.window_id = None;
                    }
                    Task::none()
                }
            }
            Message::OverlayWindowOpened(id) => {
                if self.screen_overlay.window_id == Some(id) {
                    window::enable_mouse_passthrough(id)
                } else {
                    window::close(id)
                }
            }
            Message::OverlayExpired(id) => {
                let empty = self.screen_overlay.expire(id);
                match self.screen_overlay.window_id {
                    Some(window_id) if empty => {
                        self.screen_overlay.window_id = None;
                        window::close(window_id)
                    }
                    Some(window_id) => {
                        window::resize(window_id, self.screen_overlay.window_settings().size)
                    }
                    None => Task::none(),
                }
            }
            Message::ReloadScreenOverlay => {
                let old_window = self.screen_overlay.window_id.take();
                self.screen_overlay = crate::screen_overlay::ScreenOverlay::load();
                let _ = self.screen_overlay.ingest(
                    &self.snapshot.recent_events,
                    &self.snapshot.friends,
                    &self.favorite_friend_ids,
                    &self.world_names,
                    &self.world_image_urls,
                );
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get(&self.overlay_editor_rule)
                {
                    self.overlay_editor_duration = rule
                        .duration_seconds
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    self.overlay_editor_opacity = rule
                        .opacity
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                }
                self.notice = Some(t!("screen_overlay.reloaded").to_string());
                old_window.map_or_else(Task::none, window::close)
            }
            Message::TestScreenOverlay => {
                let event = self.overlay_editor_rule.clone();
                let toast = self.screen_overlay.preview(&event);
                self.display_overlay_toasts(vec![toast])
            }
            Message::ToggleScreenOverlay => {
                self.screen_overlay.config.enabled = !self.screen_overlay.config.enabled;
                Task::none()
            }
            Message::SelectOverlayRule(event) => {
                self.overlay_editor_rule = event;
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get(&self.overlay_editor_rule)
                {
                    self.overlay_editor_duration = rule
                        .duration_seconds
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                    self.overlay_editor_opacity = rule
                        .opacity
                        .map(|value| value.to_string())
                        .unwrap_or_default();
                }
                Task::none()
            }
            Message::ToggleOverlayRule => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.enabled = !rule.enabled;
                }
                Task::none()
            }
            Message::OverlayRuleTitleChanged(value) => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.title = value;
                }
                Task::none()
            }
            Message::OverlayRuleBodyChanged(value) => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.body = value;
                }
                Task::none()
            }
            Message::OverlayRuleAccentChanged(value) => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.accent = value;
                }
                Task::none()
            }
            Message::OverlayRuleDurationChanged(value) => {
                self.overlay_editor_duration = value;
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    if self.overlay_editor_duration.trim().is_empty() {
                        rule.duration_seconds = None;
                    } else if let Ok(duration) = self.overlay_editor_duration.parse::<f32>() {
                        rule.duration_seconds = Some(duration.clamp(1.0, 30.0));
                    }
                }
                Task::none()
            }
            Message::OverlayRuleOpacityChanged(value) => {
                self.overlay_editor_opacity = value;
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    if self.overlay_editor_opacity.trim().is_empty() {
                        rule.opacity = None;
                    } else if let Ok(opacity) = self.overlay_editor_opacity.parse::<f32>() {
                        rule.opacity = Some(opacity.clamp(0.15, 1.0));
                    }
                }
                Task::none()
            }
            Message::ToggleOverlayRuleProfilePicture => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.show_profile_picture = !rule.show_profile_picture;
                }
                Task::none()
            }
            Message::ToggleOverlayRuleWorldPicture => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.show_world_picture = !rule.show_world_picture;
                }
                Task::none()
            }
            Message::OverlayRuleEveryone => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.selected_users.clear();
                    rule.favorites_only = false;
                }
                Task::none()
            }
            Message::OverlayRuleFavorites => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.selected_users.clear();
                    rule.favorites_only = true;
                }
                Task::none()
            }
            Message::ToggleOverlayRuleUser(user_id) => {
                if let Some(rule) = self
                    .screen_overlay
                    .config
                    .rules
                    .get_mut(&self.overlay_editor_rule)
                {
                    rule.favorites_only = false;
                    if rule.selected_users.is_empty() {
                        rule.selected_users.insert(user_id);
                    } else if !rule.selected_users.remove(&user_id) {
                        rule.selected_users.insert(user_id);
                    }
                }
                Task::none()
            }
            Message::SaveScreenOverlayEditor => {
                match self.screen_overlay.save_config() {
                    Ok(()) => self.notice = Some(t!("screen_overlay.saved").to_string()),
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::BootFinished(result) => {
                self.loading = false;
                match result {
                    Ok((sessions, session)) => {
                        self.saved_sessions = sessions;
                        let authenticated = session.is_some();
                        self.session = session;
                        if authenticated {
                            return Task::batch([
                                self.load_friend_favorites(),
                                self.load_saved_sessions(),
                                self.load_missing_thumbnails(),
                            ]);
                        }
                        return self.load_missing_thumbnails();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::UsernameChanged(value) => {
                self.username = value;
                Task::none()
            }
            Message::PasswordChanged(value) => {
                self.password = value;
                Task::none()
            }
            Message::Login => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                let username = self.username.clone();
                let password = self.password.clone();
                self.loading = true;
                self.error = None;
                Task::perform(
                    async move {
                        backend
                            .login(&username, &password)
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::LoginFinished,
                )
            }
            Message::LoginFinished(result) => {
                self.loading = false;
                match result {
                    Ok(LoginOutcome::Authenticated(session)) => {
                        self.session = Some(session);
                        self.password.clear();
                        return Task::batch([
                            self.load_friend_favorites(),
                            self.load_saved_sessions(),
                            self.load_missing_thumbnails(),
                        ]);
                    }
                    Ok(LoginOutcome::TwoFactorRequired(methods)) => {
                        self.selected_two_factor = methods.first().cloned();
                        self.two_factor_methods = methods;
                    }
                    Ok(LoginOutcome::InvalidCredentials) => {
                        self.error = Some("Invalid credentials".to_string());
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::TwoFactorCodeChanged(value) => {
                self.two_factor_code = value;
                Task::none()
            }
            Message::SelectTwoFactor(method) => {
                self.selected_two_factor = Some(method);
                Task::none()
            }
            Message::VerifyTwoFactor => {
                let (Some(backend), Some(method)) =
                    (self.backend.clone(), self.selected_two_factor.clone())
                else {
                    return Task::none();
                };
                let code = self.two_factor_code.clone();
                self.loading = true;
                Task::perform(
                    async move {
                        backend
                            .verify_two_factor(method, &code)
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::LoginFinished,
                )
            }
            Message::Logout => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                self.account_menu_open = false;
                self.loading = true;
                Task::perform(
                    async move { backend.logout().await.map_err(|error| error.to_string()) },
                    Message::LogoutFinished,
                )
            }
            Message::LogoutFinished(result) => {
                self.loading = false;
                match result {
                    Ok(()) => {
                        let overlay_window = self.screen_overlay.window_id.take();
                        self.screen_overlay.reset();
                        self.session = None;
                        self.snapshot = AppSnapshot::default();
                        self.results.clear();
                        self.selected_item = None;
                        self.selected_detail = None;
                        self.user_relations = UserRelations::default();
                        self.thumbnails.clear();
                        self.pending_thumbnails.clear();
                        self.favorite_friend_ids.clear();
                        self.favorite_friend_groups.clear();
                        self.two_factor_methods.clear();
                        return Task::batch([
                            self.load_saved_sessions(),
                            overlay_window.map_or_else(Task::none, window::close),
                        ]);
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::Navigate(page) => {
                self.page = page;
                self.search_overlay_open = false;
                self.results.clear();
                self.selected_item = None;
                self.selected_detail = None;
                self.user_relations = UserRelations::default();
                self.error = None;
                self.reset_render_windows();
                self.prune_thumbnail_cache();
                let page_task = self.load_page();
                if page == Page::Friends {
                    Task::batch([page_task, self.load_friend_favorites()])
                } else {
                    page_task
                }
            }
            Message::SearchChanged(value) => {
                self.search = value;
                self.search_overlay_open = !self.search.trim().is_empty();
                Task::none()
            }
            Message::SubmitSearch => {
                self.search_overlay_open = false;
                if self.page.is_searchable() {
                    self.reset_render_windows();
                    self.prune_thumbnail_cache();
                    self.load_page()
                } else {
                    self.page = Page::Users;
                    self.results.clear();
                    self.selected_item = None;
                    self.selected_detail = None;
                    self.user_relations = UserRelations::default();
                    self.error = None;
                    self.reset_render_windows();
                    self.prune_thumbnail_cache();
                    self.load_page()
                }
            }
            Message::SearchPage(page) => {
                self.page = page;
                self.search_overlay_open = false;
                self.results.clear();
                self.selected_item = None;
                self.selected_detail = None;
                self.user_relations = UserRelations::default();
                self.error = None;
                self.reset_render_windows();
                self.prune_thumbnail_cache();
                self.load_page()
            }
            Message::CloseSearchOverlay => {
                self.search_overlay_open = false;
                Task::none()
            }
            Message::Refresh => {
                self.reset_render_windows();
                self.prune_thumbnail_cache();
                let page_task = self.load_page();
                if self.page == Page::Friends {
                    Task::batch([page_task, self.load_friend_favorites()])
                } else {
                    page_task
                }
            }
            Message::ResultsLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(values) => {
                        self.results = values.into_iter().map(result_item).collect();
                        self.notice = Some(format!("{} items loaded", self.results.len()));
                        return self.load_missing_thumbnails();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::FavoriteFriendsLoaded(result) => {
                match result {
                    Ok((ids, groups)) => {
                        self.favorite_friend_ids = ids;
                        self.favorite_friend_groups = groups;
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::ResourceScrolled(offset) => {
                if offset < INFINITE_SCROLL_THRESHOLD
                    || self.resource_row_limit >= self.results.len()
                {
                    return Task::none();
                }
                self.resource_row_limit =
                    (self.resource_row_limit + RESOURCE_ROW_INCREMENT).min(self.results.len());
                self.load_missing_thumbnails()
            }
            Message::FriendSidebarScrolled(offset) => {
                if offset < INFINITE_SCROLL_THRESHOLD {
                    return Task::none();
                }
                let mut changed = false;
                for section in FriendSection::ALL {
                    if self.collapsed_friend_sections.contains(&section) {
                        continue;
                    }
                    let count = self.friend_section_count(section);
                    let limit = self.friend_section_limit(section);
                    if limit < count {
                        self.friend_section_row_limits
                            .insert(section, (limit + FRIEND_SECTION_ROW_INCREMENT).min(count));
                        changed = true;
                    }
                }
                if changed {
                    self.load_missing_thumbnails()
                } else {
                    Task::none()
                }
            }
            Message::RelationScrolled(offset) => {
                let total = self.current_relation_count();
                if offset < INFINITE_SCROLL_THRESHOLD || self.relation_row_limit >= total {
                    return Task::none();
                }
                self.relation_row_limit =
                    (self.relation_row_limit + RELATION_ROW_INCREMENT).min(total);
                self.load_missing_thumbnails()
            }
            Message::ToggleFriendSection(section) => {
                if !self.collapsed_friend_sections.remove(&section) {
                    self.collapsed_friend_sections.insert(section);
                }
                self.load_missing_thumbnails()
            }
            Message::ToggleNavCategory(category) => {
                if !self.collapsed_nav_categories.remove(&category) {
                    self.collapsed_nav_categories.insert(category);
                }
                Task::none()
            }
            Message::OpenPanelSelector => {
                self.panel_selector_open = true;
                Task::none()
            }
            Message::ClosePanelSelector => {
                self.panel_selector_open = false;
                Task::none()
            }
            Message::ToggleNavEditMode => {
                self.nav_edit_mode = !self.nav_edit_mode;
                Task::none()
            }
            Message::ToggleAccountMenu => {
                self.account_menu_open = !self.account_menu_open;
                Task::none()
            }
            Message::CloseAccountMenu => {
                self.account_menu_open = false;
                Task::none()
            }
            Message::OpenSettings => {
                self.account_menu_open = false;
                self.settings_open = true;
                self.load_saved_sessions()
            }
            Message::CloseSettings => {
                self.settings_open = false;
                Task::none()
            }
            Message::SavedSessionsLoaded(result) => {
                match result {
                    Ok(sessions) => {
                        self.saved_sessions = sessions;
                        return self.load_missing_thumbnails();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::SwitchSession(user_id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                self.loading = true;
                self.error = None;
                self.notice = None;
                self.account_menu_open = false;
                Task::perform(
                    async move {
                        backend
                            .switch_session(&user_id)
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::SwitchSessionFinished,
                )
            }
            Message::SwitchSessionFinished(result) => {
                self.loading = false;
                match result {
                    Ok(Some(session)) => {
                        self.session = Some(session);
                        self.results.clear();
                        self.selected_item = None;
                        self.selected_detail = None;
                        self.user_relations = UserRelations::default();
                        self.thumbnails.clear();
                        self.pending_thumbnails.clear();
                        self.favorite_friend_ids.clear();
                        self.favorite_friend_groups.clear();
                        self.notice = Some(t!("account.session_switched").to_string());
                        return Task::batch([
                            self.load_friend_favorites(),
                            self.load_saved_sessions(),
                            self.load_page(),
                            self.load_missing_thumbnails(),
                        ]);
                    }
                    Ok(None) => {
                        self.error = Some(t!("account.session_not_found").to_string());
                        return self.load_saved_sessions();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::AddSession => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                self.loading = true;
                self.error = None;
                self.notice = None;
                self.account_menu_open = false;
                self.settings_open = false;
                Task::perform(
                    async move {
                        backend
                            .begin_new_session()
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::AddSessionFinished,
                )
            }
            Message::AddSessionFinished(result) => {
                self.loading = false;
                match result {
                    Ok(()) => {
                        self.session = None;
                        self.snapshot = AppSnapshot::default();
                        self.results.clear();
                        self.selected_item = None;
                        self.selected_detail = None;
                        self.user_relations = UserRelations::default();
                        self.thumbnails.clear();
                        self.pending_thumbnails.clear();
                        self.favorite_friend_ids.clear();
                        self.favorite_friend_groups.clear();
                        self.two_factor_methods.clear();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::LocaleSelected(locale) => {
                self.locale = locale;
                rust_i18n::set_locale(locale.code());
                Task::none()
            }
            Message::NewCategoryNameChanged(value) => {
                self.new_category_name = value;
                Task::none()
            }
            Message::CreateNavCategory => {
                let name = self.new_category_name.trim();
                if !name.is_empty()
                    && !self
                        .nav_categories
                        .iter()
                        .any(|category| category.name.eq_ignore_ascii_case(name))
                {
                    self.nav_categories.push(NavCategory {
                        name: name.to_string(),
                        pages: Vec::new(),
                    });
                    self.new_category_name.clear();
                }
                Task::none()
            }
            Message::AddPageToNav(category_name, page) => {
                if let Some(category) = self
                    .nav_categories
                    .iter_mut()
                    .find(|category| category.name == category_name)
                {
                    if !category.pages.contains(&page) {
                        category.pages.push(page);
                    }
                }
                Task::none()
            }
            Message::RemovePageFromNav(category_name, page) => {
                if let Some(category) = self
                    .nav_categories
                    .iter_mut()
                    .find(|category| category.name == category_name)
                {
                    category.pages.retain(|candidate| *candidate != page);
                }
                if self.page == page
                    && !self
                        .nav_categories
                        .iter()
                        .any(|category| category.pages.iter().any(|candidate| *candidate == page))
                {
                    self.page = Page::Dashboard;
                    return self.load_page();
                }
                Task::none()
            }
            Message::OpenItem(index) => {
                let Some(item) = self.results.get(index).cloned() else {
                    return Task::none();
                };
                self.selected_detail = Some(item.raw.clone());
                self.selected_item = Some(item.clone());
                self.selected_page = self.page;
                self.detail_tab = DetailTab::Overview;
                self.user_relations = UserRelations::default();
                self.relation_row_limit = INITIAL_RELATION_ROWS;
                self.detail_loading = true;
                self.prune_thumbnail_cache();

                let Some(backend) = self.backend.clone() else {
                    self.detail_loading = false;
                    return Task::none();
                };
                let page = self.page;
                let detail_backend = backend.clone();
                let detail_item = item.clone();
                let detail = Task::perform(
                    async move {
                        match page {
                            Page::Users => serde_json::to_value(
                                detail_backend
                                    .api()
                                    .user(&detail_item.id)
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            Page::Avatars => serde_json::to_value(
                                detail_backend
                                    .api()
                                    .request_value(
                                        reqwest::Method::GET,
                                        &format!("avatars/{}", detail_item.id),
                                        None,
                                    )
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            Page::MyAvatars => serde_json::to_value(
                                detail_backend
                                    .api()
                                    .request_value(
                                        reqwest::Method::GET,
                                        &format!("avatars/{}", detail_item.id),
                                        None,
                                    )
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            Page::Groups => serde_json::to_value(
                                detail_backend
                                    .api()
                                    .group(&detail_item.id)
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            Page::Worlds => serde_json::to_value(
                                detail_backend
                                    .api()
                                    .world(&detail_item.id)
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            _ => Ok(detail_item.raw),
                        }
                        .map_err(|error| error.to_string())
                    },
                    Message::DetailLoaded,
                );
                if self.page == Page::Users {
                    self.relations_loading = true;
                    Task::batch([
                        detail,
                        Task::perform(
                            load_user_relations(backend, item.id),
                            Message::UserRelationsLoaded,
                        ),
                    ])
                } else {
                    self.relations_loading = false;
                    detail
                }
            }
            Message::OpenFriend(user_id) => {
                let Some(friend) = self.snapshot.friends.get(&user_id).cloned() else {
                    return Task::none();
                };
                let item = friend_result_item(&friend);
                self.selected_detail = Some(item.raw.clone());
                self.selected_item = Some(item);
                self.selected_page = Page::Friends;
                self.detail_tab = DetailTab::Overview;
                self.user_relations = UserRelations::default();
                self.relation_row_limit = INITIAL_RELATION_ROWS;
                self.detail_loading = true;
                self.relations_loading = true;
                self.prune_thumbnail_cache();
                let Some(backend) = self.backend.clone() else {
                    self.detail_loading = false;
                    self.relations_loading = false;
                    return Task::none();
                };
                let relation_backend = backend.clone();
                let relation_user_id = user_id.clone();
                Task::batch([
                    Task::perform(
                        async move {
                            serde_json::to_value(
                                backend
                                    .api()
                                    .user(&user_id)
                                    .await
                                    .map_err(|error| error.to_string())?,
                            )
                            .map_err(|error| error.to_string())
                        },
                        Message::DetailLoaded,
                    ),
                    Task::perform(
                        load_user_relations(relation_backend, relation_user_id),
                        Message::UserRelationsLoaded,
                    ),
                ])
            }
            Message::OpenRelated(item, page) => {
                self.selected_detail = Some(item.raw.clone());
                self.selected_item = Some(item.clone());
                self.selected_page = page;
                self.detail_tab = DetailTab::Overview;
                self.user_relations = UserRelations::default();
                self.relation_row_limit = INITIAL_RELATION_ROWS;
                self.relations_loading = false;
                self.detail_loading = true;
                self.prune_thumbnail_cache();
                let Some(backend) = self.backend.clone() else {
                    self.detail_loading = false;
                    return Task::none();
                };
                Task::perform(
                    async move {
                        match page {
                            Page::Users | Page::Friends => serde_json::to_value(
                                backend
                                    .api()
                                    .user(&item.id)
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            Page::Groups => serde_json::to_value(
                                backend
                                    .api()
                                    .group(&item.id)
                                    .await
                                    .map_err(|e| e.to_string())?,
                            ),
                            _ => Ok(item.raw),
                        }
                        .map_err(|error| error.to_string())
                    },
                    Message::DetailLoaded,
                )
            }
            Message::HoverItem(item_id) => {
                self.hovered_item = item_id;
                Task::none()
            }
            Message::CloseItem => {
                self.selected_item = None;
                self.selected_detail = None;
                self.detail_loading = false;
                self.relations_loading = false;
                self.user_relations = UserRelations::default();
                self.prune_thumbnail_cache();
                Task::none()
            }
            Message::DetailLoaded(result) => {
                self.detail_loading = false;
                match result {
                    Ok(detail) => self.selected_detail = Some(detail),
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::DetailTabSelected(tab) => {
                self.detail_tab = tab;
                self.relation_row_limit = INITIAL_RELATION_ROWS;
                self.load_missing_thumbnails()
            }
            Message::UserRelationsLoaded(result) => {
                self.relations_loading = false;
                match result {
                    Ok(relations) => {
                        self.user_relations = relations;
                        return self.load_missing_thumbnails();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::ThumbnailLoaded(url, result) => {
                self.pending_thumbnails.remove(&url);
                if let Ok(bytes) = result
                    && self.required_thumbnail_urls().contains(&url)
                {
                    self.thumbnails
                        .insert(url, image::Handle::from_bytes(bytes));
                }
                Task::none()
            }
            Message::WorldNameLoaded(world_id, result) => {
                self.pending_world_names.remove(&world_id);
                match result {
                    Ok((name, image_url)) => {
                        self.screen_overlay
                            .resolve_world(&world_id, &name, image_url.as_deref());
                        self.world_names.insert(world_id.clone(), name);
                        if let Some(url) = image_url {
                            self.world_image_urls.insert(world_id, url);
                        }
                        return self.load_missing_thumbnails();
                    }
                    Err(error) => {
                        tracing::warn!(world_id, %error, "world name lookup failed");
                        self.world_names.insert(world_id.clone(), world_id);
                    }
                }
                Task::none()
            }
            Message::SnapshotLoaded(snapshot) => {
                let new_toasts = self.screen_overlay.ingest(
                    &snapshot.recent_events,
                    &snapshot.friends,
                    &self.favorite_friend_ids,
                    &self.world_names,
                    &self.world_image_urls,
                );
                self.snapshot = snapshot;
                Task::batch([
                    self.load_missing_thumbnails(),
                    self.load_missing_world_names(),
                    self.display_overlay_toasts(new_toasts),
                ])
            }
            Message::Synchronize => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                self.loading = true;
                Task::perform(
                    async move {
                        backend
                            .synchronize()
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::SynchronizeFinished,
                )
            }
            Message::SynchronizeFinished(result) => {
                self.loading = false;
                match result {
                    Ok(()) => {
                        self.notice = Some("Synchronization complete".to_string());
                        return self.load_friend_favorites();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::Unfriend(user_id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        backend
                            .api()
                            .unfriend(&user_id)
                            .await
                            .map(|_| "Friend removed".to_string())
                            .map_err(|error| error.to_string())
                    },
                    Message::ActionFinished,
                )
            }
            Message::LeaveGroup(group_id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        backend
                            .api()
                            .leave_group(&group_id)
                            .await
                            .map(|_| "Group left".to_string())
                            .map_err(|error| error.to_string())
                    },
                    Message::ActionFinished,
                )
            }
            Message::RemoveFavorite(favorite_id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        backend
                            .api()
                            .remove_favorite(&favorite_id)
                            .await
                            .map(|_| "Favorite removed".to_string())
                            .map_err(|error| error.to_string())
                    },
                    Message::ActionFinished,
                )
            }
            Message::DeleteNotification(id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        backend
                            .api()
                            .delete_notification(&id)
                            .await
                            .map(|_| "Notification deleted".to_string())
                            .map_err(|error| error.to_string())
                    },
                    Message::ActionFinished,
                )
            }
            Message::ResourceAction(page, id) => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        match page {
                            Page::Users => backend
                                .api()
                                .send_friend_request(&id)
                                .await
                                .map(|_| "Friend request sent".to_string())
                                .map_err(|error| error.to_string()),
                            Page::Groups => backend
                                .api()
                                .join_group(&id)
                                .await
                                .map(|_| "Group joined".to_string())
                                .map_err(|error| error.to_string()),
                            Page::Avatars => backend
                                .api()
                                .select_avatar(&id)
                                .await
                                .map(|_| "Avatar selected".to_string())
                                .map_err(|error| error.to_string()),
                            Page::MyAvatars => backend
                                .api()
                                .select_avatar(&id)
                                .await
                                .map(|_| "Avatar selected".to_string())
                                .map_err(|error| error.to_string()),
                            Page::Inventory => backend
                                .api()
                                .consume_inventory(
                                    &id,
                                    &InventoryConsume {
                                        inventory_item_id: id.clone(),
                                        quantity: Some(1),
                                    },
                                )
                                .await
                                .map(|_| "Item consumed".to_string())
                                .map_err(|error| error.to_string()),
                            Page::Files => backend
                                .api()
                                .delete_file(&id)
                                .await
                                .map(|_| "File deleted".to_string())
                                .map_err(|error| error.to_string()),
                            Page::Props => backend
                                .api()
                                .delete_prop(&id)
                                .await
                                .map(|_| "Prop deleted".to_string())
                                .map_err(|error| error.to_string()),
                            _ => Ok("No action available".to_string()),
                        }
                    },
                    Message::ActionFinished,
                )
            }
            Message::ApiMethodChanged(method) => {
                self.api_method = method;
                Task::none()
            }
            Message::ApiPathChanged(path) => {
                self.api_path = path;
                Task::none()
            }
            Message::ApiBodyChanged(body) => {
                self.api_body = body;
                Task::none()
            }
            Message::ExecuteApi => {
                let Some(backend) = self.backend.clone() else {
                    return Task::none();
                };
                let method = self.api_method.clone();
                let path = self.api_path.clone();
                let body = self.api_body.clone();
                self.loading = true;
                Task::perform(
                    async move {
                        let method = reqwest::Method::from_bytes(method.as_bytes())
                            .map_err(|error| error.to_string())?;
                        let body = if matches!(
                            method,
                            reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::PATCH
                        ) {
                            Some(
                                serde_json::from_str::<Value>(&body)
                                    .map_err(|error| error.to_string())?,
                            )
                        } else {
                            None
                        };
                        backend
                            .api()
                            .request_value(method, &path, body.as_ref())
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::ApiExecuted,
                )
            }
            Message::ApiExecuted(result) => {
                self.loading = false;
                match result {
                    Ok(value) => {
                        self.api_response =
                            serde_json::to_string_pretty(&value).unwrap_or_default();
                    }
                    Err(error) => self.error = Some(error),
                }
                Task::none()
            }
            Message::ActionFinished(result) => {
                match result {
                    Ok(notice) => self.notice = Some(notice),
                    Err(error) => self.error = Some(error),
                }
                self.selected_item = None;
                self.selected_detail = None;
                Task::batch([self.load_page(), self.load_friend_favorites()])
            }
            Message::ClearFeedback => {
                self.error = None;
                self.notice = None;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let windows = window::close_events().map(Message::WindowClosed);
        let snapshots = self
            .backend
            .as_ref()
            .filter(|_| self.session.is_some())
            .map_or_else(Subscription::none, |backend| {
                Subscription::run_with(
                    SnapshotSubscription {
                        backend_id: Arc::as_ptr(backend) as usize,
                        receiver: backend.subscribe_state(),
                    },
                    snapshot_stream,
                )
            });
        Subscription::batch([windows, snapshots])
    }

    fn display_overlay_toasts(
        &mut self,
        toasts: Vec<crate::screen_overlay::Toast>,
    ) -> Task<Message> {
        if toasts.is_empty() {
            return Task::none();
        }
        let mut tasks = toasts
            .into_iter()
            .map(|toast| {
                Task::perform(
                    async move {
                        tokio::time::sleep(toast.duration).await;
                        toast.id
                    },
                    Message::OverlayExpired,
                )
            })
            .collect::<Vec<_>>();
        if let Some(id) = self.screen_overlay.window_id {
            tasks.push(window::resize(
                id,
                self.screen_overlay.window_settings().size,
            ));
        } else {
            let (id, open) = window::open(self.screen_overlay.window_settings());
            self.screen_overlay.window_id = Some(id);
            tasks.push(open.map(Message::OverlayWindowOpened));
        }
        Task::batch(tasks)
    }

    fn load_missing_thumbnails(&mut self) -> Task<Message> {
        let required = self.required_thumbnail_urls();
        self.prune_thumbnail_cache_to(&required);

        let urls = required
            .into_iter()
            .filter(|url| {
                !self.thumbnails.contains_key(url) && !self.pending_thumbnails.contains(url)
            })
            .collect::<HashSet<_>>();

        for url in &urls {
            self.pending_thumbnails.insert(url.clone());
        }

        Task::batch(urls.into_iter().map(|url| {
            let result_url = url.clone();
            Task::perform(download_thumbnail(url), move |result| {
                Message::ThumbnailLoaded(result_url, result)
            })
        }))
    }

    fn required_thumbnail_urls(&self) -> HashSet<String> {
        self.results
            .iter()
            .take(self.resource_row_limit)
            .filter_map(|item| item.thumbnail_url.clone())
            .chain(
                self.user_relations
                    .mutual_friends
                    .iter()
                    .chain(&self.user_relations.mutual_groups)
                    .chain(&self.user_relations.groups)
                    .take(self.relation_row_limit.min(MAX_RELATION_THUMBNAILS))
                    .filter_map(|item| item.thumbnail_url.clone()),
            )
            .chain(
                self.user_relations
                    .friend_groups
                    .iter()
                    .flat_map(|group| {
                        group
                            .raw
                            .get("members")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                    })
                    .take(self.relation_row_limit.min(MAX_RELATION_THUMBNAILS))
                    .filter_map(|member| non_empty(string_field(member, &["userIcon"]))),
            )
            .chain(self.visible_friend_thumbnail_urls())
            .chain(self.session_avatar_urls())
            .chain(self.screen_overlay.image_urls().cloned())
            .collect()
    }

    fn session_avatar_urls(&self) -> Vec<String> {
        self.session
            .as_ref()
            .and_then(|session| session.avatar_url.clone())
            .into_iter()
            .chain(
                self.saved_sessions
                    .iter()
                    .filter_map(|session| session.avatar_url.clone()),
            )
            .collect()
    }

    fn prune_thumbnail_cache(&mut self) {
        let required = self.required_thumbnail_urls();
        self.prune_thumbnail_cache_to(&required);
    }

    fn prune_thumbnail_cache_to(&mut self, required: &HashSet<String>) {
        self.thumbnails.retain(|url, _| required.contains(url));
        self.pending_thumbnails.retain(|url| required.contains(url));
    }

    fn visible_friend_thumbnail_urls(&self) -> Vec<String> {
        let mut friends = self.snapshot.friends.values().collect::<Vec<_>>();
        friends.sort_by_key(|friend| {
            (
                friend_section(friend, &self.favorite_friend_ids),
                friend
                    .display_name
                    .as_deref()
                    .unwrap_or(&friend.user_id)
                    .to_lowercase(),
            )
        });
        FriendSection::ALL
            .into_iter()
            .filter(|section| !self.collapsed_friend_sections.contains(section))
            .flat_map(|section| {
                friends
                    .iter()
                    .copied()
                    .filter(move |friend| {
                        friend_section(friend, &self.favorite_friend_ids) == section
                    })
                    .take(self.friend_section_limit(section))
                    .filter_map(|friend| friend.avatar_url.clone())
            })
            .take(MAX_FRIEND_THUMBNAILS)
            .collect()
    }

    fn reset_render_windows(&mut self) {
        self.resource_row_limit = INITIAL_RESOURCE_ROWS;
        self.relation_row_limit = INITIAL_RELATION_ROWS;
        self.friend_section_row_limits = default_friend_section_row_limits();
    }

    fn friend_section_limit(&self, section: FriendSection) -> usize {
        self.friend_section_row_limits
            .get(&section)
            .copied()
            .unwrap_or(INITIAL_FRIEND_SECTION_ROWS)
    }

    fn friend_section_count(&self, section: FriendSection) -> usize {
        self.snapshot
            .friends
            .values()
            .filter(|friend| friend_section(friend, &self.favorite_friend_ids) == section)
            .count()
    }

    fn current_relation_count(&self) -> usize {
        match self.detail_tab {
            DetailTab::MutualFriends => self.user_relations.mutual_friends.len(),
            DetailTab::MutualGroups => self.user_relations.mutual_groups.len(),
            DetailTab::Groups => self.user_relations.groups.len(),
            DetailTab::FriendGroups => self
                .user_relations
                .friend_groups
                .iter()
                .filter_map(|group| group.raw.get("members").and_then(Value::as_array))
                .map(Vec::len)
                .sum(),
            DetailTab::Overview => 0,
        }
    }

    fn load_missing_world_names(&mut self) -> Task<Message> {
        let Some(backend) = self.backend.clone() else {
            return Task::none();
        };
        let ids = self
            .snapshot
            .friends
            .values()
            .flat_map(|friend| {
                [
                    friend.world_id.as_deref(),
                    friend.location.as_deref().and_then(world_id_from_location),
                    friend
                        .traveling_to_location
                        .as_deref()
                        .and_then(world_id_from_location),
                ]
            })
            .flatten()
            .chain(
                self.snapshot
                    .recent_events
                    .iter()
                    .flat_map(|event| event_world_ids(&event.event)),
            )
            .filter(|id| {
                !self.world_names.contains_key(*id) && !self.pending_world_names.contains(*id)
            })
            .map(str::to_string)
            .collect::<HashSet<_>>();

        for id in &ids {
            self.pending_world_names.insert(id.clone());
        }
        Task::batch(ids.into_iter().map(|world_id| {
            let backend = backend.clone();
            let result_id = world_id.clone();
            Task::perform(
                async move {
                    backend
                        .api()
                        .request_value(reqwest::Method::GET, &format!("worlds/{world_id}"), None)
                        .await
                        .map_err(|error| error.to_string())
                        .and_then(|world| {
                            let name = world
                                .get("name")
                                .and_then(Value::as_str)
                                .map(str::to_string)
                                .ok_or_else(|| "world response is missing name".to_string())?;
                            let image_url = world
                                .get("thumbnailImageUrl")
                                .or_else(|| world.get("imageUrl"))
                                .and_then(Value::as_str)
                                .filter(|url| !url.is_empty())
                                .map(str::to_string);
                            Ok((name, image_url))
                        })
                },
                move |result| Message::WorldNameLoaded(result_id, result),
            )
        }))
    }

    fn load_page(&mut self) -> Task<Message> {
        let Some(backend) = self.backend.clone() else {
            return Task::none();
        };
        let page = self.page;
        let query = self.search.clone();
        self.loading = true;

        Task::perform(
            async move {
                let values = match page {
                    Page::Dashboard | Page::Friends | Page::Notifications => {
                        serde_json::to_value(Vec::<Value>::new())
                    }
                    #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
                    Page::VrOverlay => serde_json::to_value(Vec::<Value>::new()),
                    Page::Worlds => serde_json::to_value(
                        backend
                            .api()
                            .search_worlds(&WorldSearchQuery {
                                page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                search: non_empty(query),
                                ..Default::default()
                            })
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::Users => {
                        if let Some(search) = non_empty(query) {
                            serde_json::to_value(
                                backend
                                    .api()
                                    .search_users(&UserSearchQuery {
                                        page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                        search: Some(search),
                                    })
                                    .await
                                    .map_err(|error| error.to_string())?,
                            )
                        } else {
                            let mut friends = backend
                                .store()
                                .snapshot()
                                .await
                                .friends
                                .into_values()
                                .collect::<Vec<_>>();
                            friends.sort_by_key(|friend| {
                                (
                                    !friend.online,
                                    friend
                                        .display_name
                                        .as_deref()
                                        .unwrap_or(&friend.user_id)
                                        .to_lowercase(),
                                )
                            });
                            serde_json::to_value(
                                friends
                                    .into_iter()
                                    .map(|friend| friend_result_item(&friend).raw)
                                    .collect::<Vec<_>>(),
                            )
                        }
                    }
                    Page::Groups => serde_json::to_value(
                        backend
                            .api()
                            .search_groups(&GroupSearchQuery {
                                page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                query: non_empty(query),
                            })
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::Avatars => serde_json::to_value(
                        backend
                            .api()
                            .get_json_with_query::<Vec<Value>, _>(
                                "avatars",
                                &AvatarSearchQuery {
                                    page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                    search: non_empty(query),
                                    ..Default::default()
                                },
                            )
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::MyAvatars => {
                        let search = non_empty(query);
                        let user_id = backend
                            .store()
                            .snapshot()
                            .await
                            .session
                            .user_id
                            .ok_or_else(|| "A user ID is required".to_string())?;
                        let current = backend
                            .api()
                            .request_value(
                                reqwest::Method::GET,
                                &format!("users/{user_id}/avatar"),
                                None,
                            )
                            .await
                            .ok();
                        let mut avatars = backend
                            .api()
                            .get_json_with_query::<Vec<Value>, _>(
                                "avatars",
                                &OwnAvatarQuery {
                                    page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                    search,
                                    user: "me",
                                    release_status: "all",
                                    order: "descending",
                                },
                            )
                            .await
                            .map_err(|error| error.to_string())?;
                        if let Some(current) = current.filter(Value::is_object) {
                            let current_id = current.get("id").and_then(Value::as_str);
                            if current_id.is_none_or(|id| {
                                !avatars.iter().any(|avatar| {
                                    avatar.get("id").and_then(Value::as_str) == Some(id)
                                })
                            }) {
                                avatars.insert(0, current);
                            }
                        }
                        serde_json::to_value(avatars)
                    }
                    Page::Inventory => serde_json::to_value(
                        backend
                            .api()
                            .inventory(&InventoryQuery {
                                page: PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                search: non_empty(query),
                                ..Default::default()
                            })
                            .await
                            .map_err(|error| error.to_string())?
                            .items,
                    ),
                    Page::Calendar => serde_json::to_value(
                        backend
                            .api()
                            .calendar(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?
                            .items,
                    ),
                    Page::Economy => serde_json::to_value(
                        backend
                            .api()
                            .stores(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::Files => serde_json::to_value(
                        backend
                            .api()
                            .files(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?
                            .items,
                    ),
                    Page::Jams => serde_json::to_value(
                        backend
                            .api()
                            .jams(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?
                            .items,
                    ),
                    Page::Props => serde_json::to_value(
                        backend
                            .api()
                            .props(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?
                            .items,
                    ),
                    Page::Favorites => serde_json::to_value(
                        load_favorite_resources(backend.clone(), non_empty(query)).await?,
                    ),
                    Page::Instances => serde_json::to_value(
                        backend
                            .api()
                            .recent_locations(&PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT))
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::Invites => {
                        let snapshot = backend.store().snapshot().await;
                        let user_id = non_empty(query)
                            .or(snapshot.session.user_id)
                            .ok_or_else(|| "A user ID is required".to_string())?;
                        serde_json::to_value(
                            backend
                                .api()
                                .invite_messages(&user_id, &InviteMessageType::Message)
                                .await
                                .map_err(|error| error.to_string())?
                                .messages,
                        )
                    }
                    Page::Moderation => serde_json::to_value(
                        backend
                            .api()
                            .player_moderations()
                            .await
                            .map_err(|error| error.to_string())?,
                    ),
                    Page::Prints => {
                        let snapshot = backend.store().snapshot().await;
                        let user_id = non_empty(query)
                            .or(snapshot.session.user_id)
                            .ok_or_else(|| "A user ID is required".to_string())?;
                        serde_json::to_value(
                            backend
                                .api()
                                .user_prints(
                                    &user_id,
                                    &PaginationQuery::new().limit(RESOURCE_PAGE_LIMIT),
                                )
                                .await
                                .map_err(|error| error.to_string())?
                                .items,
                        )
                    }
                    Page::System => {
                        let config = backend
                            .api()
                            .config()
                            .await
                            .map_err(|error| error.to_string())?;
                        serde_json::to_value(vec![config])
                    }
                    Page::Api => serde_json::to_value(Vec::<Value>::new()),
                }
                .map_err(|error| error.to_string())?;

                Ok::<_, String>(values.as_array().cloned().unwrap_or_default())
            },
            Message::ResultsLoaded,
        )
    }

    fn load_friend_favorites(&self) -> Task<Message> {
        let Some(backend) = self.backend.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let favorite_groups = backend
                    .api()
                    .favorite_groups()
                    .await
                    .map_err(|error| error.to_string())?
                    .groups
                    .into_iter()
                    .filter(|group| {
                        matches!(
                            group.favorite_type,
                            crate::models::favorite::FavoriteType::Friend
                        )
                    })
                    .collect::<Vec<_>>();
                let mut ids = HashSet::new();
                let mut favorites_by_group = HashMap::<String, HashSet<String>>::new();
                let mut offset = 0;
                loop {
                    let favorites = backend
                        .api()
                        .favorites(&FavoritesQuery {
                            page: PaginationQuery::new().limit(100).offset(offset),
                            r#type: Some(crate::models::favorite::FavoriteType::Friend),
                            tag: None,
                        })
                        .await
                        .map_err(|error| error.to_string())?;
                    let count = favorites.items.len();
                    for favorite in favorites.items {
                        ids.insert(favorite.favorite_id.clone());
                        for tag in favorite.tags {
                            favorites_by_group
                                .entry(tag)
                                .or_default()
                                .insert(favorite.favorite_id.clone());
                        }
                    }
                    if count < 100 {
                        break;
                    }
                    offset += 100;
                }
                let mut matched_tags = HashSet::new();
                let mut groups = favorite_groups
                    .into_iter()
                    .map(|group| {
                        let mut friend_ids = HashSet::new();
                        for (tag, tagged_ids) in &favorites_by_group {
                            if favorite_group_matches_tag(&group, tag) {
                                matched_tags.insert(tag.clone());
                                friend_ids.extend(tagged_ids.iter().cloned());
                            }
                        }
                        FavoriteFriendGroup {
                            friend_ids,
                            display_name: if group.display_name.trim().is_empty() {
                                group.name.clone()
                            } else {
                                group.display_name
                            },
                            name: group.name,
                        }
                    })
                    .collect::<Vec<_>>();
                groups.extend(
                    favorites_by_group
                        .into_iter()
                        .filter(|(tag, _)| !matched_tags.contains(tag))
                        .map(|(tag, friend_ids)| FavoriteFriendGroup {
                            friend_ids,
                            display_name: tag.clone(),
                            name: tag,
                        }),
                );
                groups.sort_by(|left, right| left.name.cmp(&right.name));
                Ok((ids, groups))
            },
            Message::FavoriteFriendsLoaded,
        )
    }

    fn load_saved_sessions(&self) -> Task<Message> {
        let Some(backend) = self.backend.clone() else {
            return Task::none();
        };
        Task::perform(
            async move { backend.saved_sessions().map_err(|error| error.to_string()) },
            Message::SavedSessionsLoaded,
        )
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if self.screen_overlay.window_id == Some(id) {
            self.screen_overlay.view(&self.thumbnails)
        } else {
            container(self.view())
                .width(Fill)
                .height(Fill)
                .style(|theme: &Theme| {
                    container::Style::default().background(theme.palette().background)
                })
                .into()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.session.is_none() {
            return self.auth_view();
        }

        let sidebar = self.sidebar();
        let body = column![rule::horizontal(1), self.feedback(), self.page_view()].height(Fill);
        let body: Element<'_, Message> =
            if self.search_overlay_open && !self.search.trim().is_empty() {
                stack![body, opaque(self.search_overlay())].into()
            } else {
                body.into()
            };
        let content = column![self.topbar(), body].height(Fill);

        let base: Element<'_, Message> = row![sidebar, content, self.friends_sidebar()]
            .height(Fill)
            .into();
        let base = if self.selected_item.is_some() {
            stack![
                base,
                opaque(
                    container(self.detail_modal())
                        .width(Fill)
                        .height(Fill)
                        .center_x(Fill)
                        .center_y(Fill)
                )
            ]
            .into()
        } else {
            base
        };

        let base = if self.panel_selector_open {
            stack![
                base,
                opaque(
                    container(self.panel_selector_modal())
                        .width(Fill)
                        .height(Fill)
                        .center_x(Fill)
                        .center_y(Fill)
                )
            ]
            .into()
        } else {
            base
        };

        if self.settings_open {
            stack![
                base,
                opaque(
                    container(self.settings_modal())
                        .width(Fill)
                        .height(Fill)
                        .center_x(Fill)
                        .center_y(Fill)
                )
            ]
            .into()
        } else {
            base
        }
    }

    fn search_overlay(&self) -> Element<'_, Message> {
        let filters = Page::SEARCH_FILTERS
            .chunks(2)
            .fold(column![].spacing(8), |column, chunk| {
                let row = chunk.iter().copied().fold(row![].spacing(8), |row, page| {
                    row.push(
                        button(text(page.label()).size(14))
                            .width(Fill)
                            .padding([9, 12])
                            .on_press(Message::SearchPage(page))
                            .style(if self.page == page {
                                button::primary
                            } else {
                                button::secondary
                            }),
                    )
                });
                column.push(row)
            });

        let panel = container(
            column![
                row![
                    text(t!("search.filters").to_string()).size(15),
                    Space::new().width(Fill),
                    button(text(t!("actions.dismiss").to_string()).size(14))
                        .on_press(Message::CloseSearchOverlay)
                        .style(button::text)
                ]
                .align_y(iced::Center),
                text(&self.search).size(13),
                filters
            ]
            .spacing(10),
        )
        .width(Length::Fixed(430.0))
        .padding(14)
        .style(search_modal_style);

        container(row![Space::new().width(Fill), panel].padding([12, 20]))
            .width(Fill)
            .height(Fill)
            .style(search_dim_style)
            .into()
    }

    fn panel_selector_modal(&self) -> Element<'_, Message> {
        let new_category_placeholder = t!("nav.new_category").to_string();
        let target_categories =
            self.nav_categories
                .iter()
                .fold(column![].spacing(10), |column, category| {
                    let pages = Page::ALL
                        .chunks(2)
                        .fold(column![].spacing(6), |column, chunk| {
                            let row = chunk.iter().copied().fold(row![].spacing(6), |row, page| {
                                let contains = category.pages.contains(&page);
                                row.push(
                                    button(text(page.label()))
                                        .width(Fill)
                                        .on_press(if contains {
                                            Message::RemovePageFromNav(category.name.clone(), page)
                                        } else {
                                            Message::AddPageToNav(category.name.clone(), page)
                                        })
                                        .style(if contains {
                                            button::primary
                                        } else {
                                            button::secondary
                                        })
                                        .padding([7, 10]),
                                )
                            });
                            column.push(row)
                        });
                    column.push(
                        column![
                            row![
                                text(&category.name).size(13),
                                Space::new().width(Fill),
                                text(
                                    t!("nav.panel_count", count = category.pages.len()).to_string()
                                )
                                .size(11)
                            ]
                            .align_y(iced::Center),
                            pages
                        ]
                        .spacing(7),
                    )
                });

        container(
            column![
                row![
                    text(t!("nav.select_panel_content").to_string()).size(18),
                    Space::new().width(Fill),
                    button(text(t!("actions.close").to_string()).size(12))
                        .on_press(Message::ClosePanelSelector)
                        .style(button::text)
                ]
                .align_y(iced::Center),
                text(t!("nav.panel_selector_help").to_string()).size(11),
                rule::horizontal(1),
                row![
                    text_input(&new_category_placeholder, &self.new_category_name)
                        .on_input(Message::NewCategoryNameChanged)
                        .on_submit(Message::CreateNavCategory)
                        .padding(9),
                    button(text(t!("actions.create").to_string()).size(14))
                        .on_press(Message::CreateNavCategory)
                        .style(button::secondary)
                ]
                .spacing(8),
                scrollable(target_categories)
                    .spacing(SCROLLBAR_SPACING)
                    .height(Length::Fixed(420.0)),
                row![
                    Space::new().width(Fill),
                    button(text(t!("actions.close").to_string()).size(14))
                        .on_press(Message::ClosePanelSelector)
                        .style(button::secondary)
                ]
            ]
            .spacing(12),
        )
        .width(Length::Fixed(620.0))
        .max_width(620.0)
        .padding(18)
        .style(container::bordered_box)
        .into()
    }

    fn settings_modal(&self) -> Element<'_, Message> {
        let sessions: Element<'_, Message> = if self.saved_sessions.is_empty() {
            container(text(t!("settings.no_sessions").to_string()).size(13))
                .height(Length::Fixed(80.0))
                .center_y(Length::Fixed(80.0))
                .into()
        } else {
            self.saved_sessions
                .iter()
                .fold(column![].spacing(8), |column, session| {
                    let active = session.active
                        || self
                            .session
                            .as_ref()
                            .is_some_and(|active| active.user_id == session.user_id);
                    let action: Element<'_, Message> = if active {
                        container(text(t!("account.active").to_string()).size(12))
                            .padding([6, 10])
                            .style(container::bordered_box)
                            .into()
                    } else {
                        button(text(t!("account.switch").to_string()).size(13))
                            .on_press(Message::SwitchSession(session.user_id.clone()))
                            .style(button::secondary)
                            .padding([7, 10])
                            .into()
                    };
                    column.push(
                        container(
                            row![
                                account_avatar(
                                    &session.display_name,
                                    session.avatar_url.as_deref(),
                                    &self.thumbnails
                                ),
                                column![
                                    text(session.display_name.clone()).size(14),
                                    text(session.user_id.clone()).size(10)
                                ]
                                .spacing(2)
                                .width(Fill),
                                action
                            ]
                            .spacing(10)
                            .align_y(iced::Center),
                        )
                        .padding(10)
                        .width(Fill)
                        .style(container::bordered_box),
                    )
                })
                .into()
        };

        let locale_controls = Locale::ALL
            .into_iter()
            .fold(row![].spacing(8), |row, locale| {
                row.push(
                    button(text(locale.label()).size(13))
                        .on_press(Message::LocaleSelected(locale))
                        .style(if self.locale == locale {
                            button::primary
                        } else {
                            button::secondary
                        })
                        .padding([8, 14]),
                )
            });

        #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
        let vr_settings: Element<'_, Message> = column![
            rule::horizontal(1),
            text(t!("vr.title").to_string()).size(15),
            text(vr_build_summary()).size(12),
            button(text(t!("vr.open_page").to_string()).size(13))
                .on_press(Message::Navigate(Page::VrOverlay))
                .style(button::secondary)
                .padding([8, 12])
        ]
        .spacing(9)
        .into();

        #[cfg(not(all(feature = "vr-overlay", not(target_os = "macos"))))]
        let vr_settings: Element<'_, Message> = Space::new().height(0).into();

        let screen_overlay_options = self.screen_overlay_options();

        container(
            column![
                row![
                    text(t!("settings.title").to_string()).size(18),
                    Space::new().width(Fill),
                    button(text(t!("actions.close").to_string()).size(12))
                        .on_press(Message::CloseSettings)
                        .style(button::text)
                ]
                .align_y(iced::Center),
                rule::horizontal(1),
                scrollable(
                    column![
                        text(t!("settings.sessions").to_string()).size(15),
                        sessions,
                        button(text(t!("account.add_session").to_string()).size(13))
                            .on_press(Message::AddSession)
                            .style(button::secondary)
                            .padding([8, 12]),
                        Space::new().height(6),
                        text(t!("settings.localization").to_string()).size(15),
                        locale_controls,
                        screen_overlay_options,
                        vr_settings
                    ]
                    .spacing(12)
                )
                .spacing(SCROLLBAR_SPACING)
            ]
            .spacing(12),
        )
        .width(Length::Fixed(720.0))
        .height(Fill)
        .max_width(720.0)
        .max_height(680.0)
        .padding(18)
        .style(container::bordered_box)
        .into()
    }

    fn screen_overlay_options(&self) -> Element<'_, Message> {
        let selected_event = self.overlay_editor_rule.as_str();
        let title_placeholder = t!("screen_overlay.title_template").to_string();
        let body_placeholder = t!("screen_overlay.body_template").to_string();
        let duration_placeholder = t!("screen_overlay.duration").to_string();
        let opacity_placeholder = t!("screen_overlay.opacity").to_string();
        let rule_buttons = self
            .screen_overlay
            .config
            .rules
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .chunks(3)
            .fold(column![].spacing(6), |rows, events| {
                rows.push(events.iter().fold(row![].spacing(6), |row, event| {
                    row.push(
                        button(text(event.clone()).size(11))
                            .on_press(Message::SelectOverlayRule(event.clone()))
                            .style(if event == selected_event {
                                button::primary
                            } else {
                                button::secondary
                            })
                            .padding([6, 8])
                            .width(Fill),
                    )
                }))
            });

        let Some(event_rule) = self.screen_overlay.config.rules.get(selected_event) else {
            return Space::new().height(0).into();
        };
        let global_label = if self.screen_overlay.config.enabled {
            t!("screen_overlay.disable").to_string()
        } else {
            t!("screen_overlay.enable").to_string()
        };
        let rule_label = if event_rule.enabled {
            t!("screen_overlay.disable_event").to_string()
        } else {
            t!("screen_overlay.enable_event").to_string()
        };

        let people_editor: Element<'_, Message> = if selected_event.starts_with("friend-") {
            let mut friends = self.snapshot.friends.values().collect::<Vec<_>>();
            friends.sort_by_key(|friend| {
                (
                    !self.favorite_friend_ids.contains(&friend.user_id),
                    friend
                        .display_name
                        .as_deref()
                        .unwrap_or(&friend.user_id)
                        .to_lowercase(),
                )
            });
            let friend_rows = friends
                .into_iter()
                .fold(column![].spacing(4), |rows, friend| {
                    let selected = event_rule.selected_users.contains(&friend.user_id);
                    let label = friend
                        .display_name
                        .as_deref()
                        .unwrap_or(&friend.user_id)
                        .to_string();
                    rows.push(
                        button(text(label).size(12))
                            .on_press(Message::ToggleOverlayRuleUser(friend.user_id.clone()))
                            .style(if selected {
                                button::primary
                            } else {
                                button::secondary
                            })
                            .padding([6, 8])
                            .width(Fill),
                    )
                });
            column![
                text(t!("screen_overlay.people").to_string()).size(13),
                row![
                    button(text(t!("screen_overlay.favorites_only").to_string()).size(12))
                        .on_press(Message::OverlayRuleFavorites)
                        .style(
                            if event_rule.favorites_only && event_rule.selected_users.is_empty() {
                                button::primary
                            } else {
                                button::secondary
                            }
                        ),
                    button(text(t!("screen_overlay.everyone").to_string()).size(12))
                        .on_press(Message::OverlayRuleEveryone)
                        .style(
                            if !event_rule.favorites_only && event_rule.selected_users.is_empty() {
                                button::primary
                            } else {
                                button::secondary
                            }
                        ),
                ]
                .spacing(8),
                scrollable(friend_rows)
                    .spacing(SCROLLBAR_SPACING)
                    .height(Length::Fixed(130.0)),
            ]
            .spacing(6)
            .into()
        } else {
            Space::new().height(0).into()
        };

        column![
            rule::horizontal(1),
            row![
                text(t!("screen_overlay.title").to_string()).size(16),
                Space::new().width(Fill),
                button(text(global_label).size(12))
                    .on_press(Message::ToggleScreenOverlay)
                    .style(button::secondary),
                button(text(t!("screen_overlay.test").to_string()).size(12))
                    .on_press(Message::TestScreenOverlay)
                    .style(button::primary),
            ]
            .spacing(8)
            .align_y(iced::Center),
            text(t!("screen_overlay.description").to_string()).size(12),
            rule_buttons,
            row![
                text(selected_event.to_string()).size(14),
                Space::new().width(Fill),
                button(text(rule_label).size(12))
                    .on_press(Message::ToggleOverlayRule)
                    .style(if event_rule.enabled {
                        button::primary
                    } else {
                        button::secondary
                    }),
            ]
            .align_y(iced::Center),
            text_input(&title_placeholder, &event_rule.title)
                .on_input(Message::OverlayRuleTitleChanged)
                .padding(8),
            text_input(&body_placeholder, &event_rule.body)
                .on_input(Message::OverlayRuleBodyChanged)
                .padding(8),
            row![
                text_input("#6C8CFF", &event_rule.accent)
                    .on_input(Message::OverlayRuleAccentChanged)
                    .padding(8),
                text_input(&duration_placeholder, &self.overlay_editor_duration)
                    .on_input(Message::OverlayRuleDurationChanged)
                    .padding(8),
                text_input(&opacity_placeholder, &self.overlay_editor_opacity)
                    .on_input(Message::OverlayRuleOpacityChanged)
                    .padding(8),
            ]
            .spacing(8),
            row![
                button(
                    text(if event_rule.show_profile_picture {
                        t!("screen_overlay.profile_picture_on").to_string()
                    } else {
                        t!("screen_overlay.profile_picture_off").to_string()
                    })
                    .size(12)
                )
                .on_press(Message::ToggleOverlayRuleProfilePicture)
                .style(if event_rule.show_profile_picture {
                    button::primary
                } else {
                    button::secondary
                }),
                button(
                    text(if event_rule.show_world_picture {
                        t!("screen_overlay.world_picture_on").to_string()
                    } else {
                        t!("screen_overlay.world_picture_off").to_string()
                    })
                    .size(12)
                )
                .on_press(Message::ToggleOverlayRuleWorldPicture)
                .style(if event_rule.show_world_picture {
                    button::primary
                } else {
                    button::secondary
                }),
            ]
            .spacing(8),
            people_editor,
            row![
                button(text(t!("screen_overlay.save").to_string()).size(12))
                    .on_press(Message::SaveScreenOverlayEditor)
                    .style(button::primary),
                button(text(t!("screen_overlay.reload").to_string()).size(12))
                    .on_press(Message::ReloadScreenOverlay)
                    .style(button::secondary),
                Space::new().width(Fill),
                text(self.screen_overlay.config_path.display().to_string()).size(10),
            ]
            .spacing(8)
            .align_y(iced::Center),
            text(t!("screen_overlay.performance_hint").to_string()).size(11),
        ]
        .spacing(8)
        .into()
    }

    fn auth_view(&self) -> Element<'_, Message> {
        let title = text("VRCX - BIR").size(34);
        let subtitle = text(t!("auth.subtitle").to_string()).size(16);
        let username_placeholder = t!("auth.username").to_string();
        let password_placeholder = t!("auth.password").to_string();
        let verification_placeholder = t!("auth.verification_code").to_string();

        let form: Element<'_, Message> = if self.two_factor_methods.is_empty() {
            column![
                text_input(&username_placeholder, &self.username)
                    .on_input(Message::UsernameChanged)
                    .padding(12),
                text_input(&password_placeholder, &self.password)
                    .on_input(Message::PasswordChanged)
                    .secure(true)
                    .padding(12),
                button(text(t!("auth.sign_in").to_string()).size(14))
                    .on_press_maybe((!self.loading).then_some(Message::Login))
                    .style(button::primary)
                    .padding([10, 18]),
            ]
            .spacing(12)
            .into()
        } else {
            let methods =
                self.two_factor_methods
                    .iter()
                    .cloned()
                    .fold(row![].spacing(8), |row, method| {
                        let selected = self.selected_two_factor.as_ref() == Some(&method);
                        row.push(
                            button(text(two_factor_label(&method)).size(14))
                                .on_press(Message::SelectTwoFactor(method))
                                .style(if selected {
                                    button::primary
                                } else {
                                    button::secondary
                                }),
                        )
                    });
            column![
                text(t!("auth.two_factor").to_string()).size(18),
                methods,
                text_input(&verification_placeholder, &self.two_factor_code)
                    .on_input(Message::TwoFactorCodeChanged)
                    .padding(12),
                button(text(t!("auth.verify").to_string()).size(14))
                    .on_press_maybe((!self.loading).then_some(Message::VerifyTwoFactor))
                    .style(button::primary)
                    .padding([10, 18]),
            ]
            .spacing(12)
            .into()
        };

        let saved_sessions: Element<'_, Message> = if self.saved_sessions.is_empty() {
            Space::new().height(0).into()
        } else {
            let sessions =
                self.saved_sessions
                    .iter()
                    .fold(column![].spacing(7), |column, session| {
                        column.push(
                            button(
                                row![
                                    account_avatar(
                                        &session.display_name,
                                        session.avatar_url.as_deref(),
                                        &self.thumbnails
                                    ),
                                    column![
                                        text(session.display_name.clone()).size(14),
                                        text(session.user_id.clone()).size(10)
                                    ]
                                    .spacing(2)
                                    .width(Fill),
                                    text(t!("account.switch").to_string()).size(12)
                                ]
                                .spacing(10)
                                .align_y(iced::Center),
                            )
                            .on_press_maybe(
                                (!self.loading)
                                    .then_some(Message::SwitchSession(session.user_id.clone())),
                            )
                            .style(button::secondary)
                            .padding([8, 10])
                            .width(Fill),
                        )
                    });

            column![
                text(t!("settings.sessions").to_string()).size(14),
                sessions,
                rule::horizontal(1)
            ]
            .spacing(9)
            .into()
        };

        let panel = container(column![
            title,
            subtitle,
            Space::new().height(18),
            self.feedback(),
            saved_sessions,
            form
        ])
        .width(Length::Fixed(390.0))
        .padding(28)
        .style(container::bordered_box);

        container(panel).center(Fill).into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let nav = self
            .nav_categories
            .iter()
            .fold(column![].spacing(8), |column, category| {
                let collapsed = self.collapsed_nav_categories.contains(&category.name);
                let rows: Element<'_, Message> = if collapsed {
                    Space::new().height(0).into()
                } else if category.pages.is_empty() {
                    text(t!("nav.empty").to_string()).size(11).into()
                } else {
                    category
                        .pages
                        .iter()
                        .copied()
                        .fold(column![].spacing(4), |rows, page| {
                            let remove_button: Element<'_, Message> = if self.nav_edit_mode {
                                button(text(t!("actions.remove").to_string()).size(11))
                                    .on_press(Message::RemovePageFromNav(
                                        category.name.clone(),
                                        page,
                                    ))
                                    .style(button::text)
                                    .padding([8, 6])
                                    .into()
                            } else {
                                Space::new().width(0).into()
                            };
                            rows.push(
                                row![
                                    button(text(page.label()).size(14))
                                        .width(Fill)
                                        .padding([8, 10])
                                        .style(if self.page == page {
                                            button::primary
                                        } else {
                                            button::text
                                        })
                                        .on_press(Message::Navigate(page)),
                                    remove_button
                                ]
                                .align_y(iced::Center),
                            )
                        })
                        .into()
                };
                column.push(
                    column![
                        button(
                            row![
                                text(if collapsed { ">" } else { "v" }).size(11),
                                text(&category.name).size(11),
                                Space::new().width(Fill),
                                text(category.pages.len().to_string()).size(10)
                            ]
                            .align_y(iced::Center)
                        )
                        .on_press(Message::ToggleNavCategory(category.name.clone()))
                        .style(button::text)
                        .padding([4, 2])
                        .width(Fill),
                        rows
                    ]
                    .spacing(4),
                )
            });

        let account_controls: Element<'_, Message> = if self.account_menu_open {
            column![self.account_menu(), self.account_button()]
                .spacing(10)
                .into()
        } else {
            self.account_button()
        };

        container(column![
            text("VRCX - BIR").size(24),
            Space::new().height(18),
            scrollable(nav).spacing(SCROLLBAR_SPACING).height(Fill),
            account_controls
        ])
        .width(Length::Fixed(190.0))
        .height(Fill)
        .padding(14)
        .style(container::dark)
        .into()
    }

    fn account_button(&self) -> Element<'_, Message> {
        let name = self
            .session
            .as_ref()
            .map(|session| session.display_name.as_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("VRChat");
        let avatar_url = self
            .session
            .as_ref()
            .and_then(|session| session.avatar_url.as_deref());
        button(
            row![
                account_avatar(name, avatar_url, &self.thumbnails),
                text(name.to_string()).size(15).width(Fill),
                text(if self.account_menu_open { "^" } else { "v" }).size(12)
            ]
            .spacing(10)
            .align_y(iced::Center),
        )
        .on_press(Message::ToggleAccountMenu)
        .style(button::text)
        .padding([8, 8])
        .width(Fill)
        .into()
    }

    fn account_menu(&self) -> Element<'_, Message> {
        let name = self
            .session
            .as_ref()
            .map(|session| session.display_name.as_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("VRChat");
        let avatar_url = self
            .session
            .as_ref()
            .and_then(|session| session.avatar_url.as_deref());
        let edit_label = if self.nav_edit_mode {
            t!("actions.done").to_string()
        } else {
            t!("actions.edit").to_string()
        };

        container(
            column![
                row![
                    account_avatar(name, avatar_url, &self.thumbnails),
                    column![
                        text(name.to_string()).size(15),
                        text(t!("account.connected").to_string()).size(10)
                    ]
                    .spacing(2)
                    .width(Fill),
                    button(text("x").size(12))
                        .on_press(Message::CloseAccountMenu)
                        .style(button::text)
                        .padding([4, 6])
                ]
                .spacing(10)
                .align_y(iced::Center),
                rule::horizontal(1),
                account_menu_button(
                    "⚙",
                    t!("account.settings").to_string(),
                    Message::OpenSettings
                ),
                account_menu_button("✎", edit_label, Message::ToggleNavEditMode),
                account_menu_button("▦", t!("nav.panel").to_string(), Message::OpenPanelSelector),
                rule::horizontal(1),
                account_menu_button("⇱", t!("auth.sign_out").to_string(), Message::Logout),
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Fill)
        .style(account_menu_style)
        .into()
    }

    fn friends_sidebar(&self) -> Element<'_, Message> {
        let mut friends = self.snapshot.friends.values().collect::<Vec<_>>();
        friends.sort_by_key(|friend| {
            (
                !friend.online,
                friend
                    .display_name
                    .as_deref()
                    .unwrap_or(&friend.user_id)
                    .to_lowercase(),
            )
        });
        let in_vrchat = friends
            .iter()
            .copied()
            .filter(|friend| {
                !self.favorite_friend_ids.contains(&friend.user_id)
                    && public_instance_location(friend).is_some()
            })
            .collect::<Vec<_>>();
        let private = friends
            .iter()
            .copied()
            .filter(|friend| {
                !self.favorite_friend_ids.contains(&friend.user_id)
                    && friend_is_in_vrchat(friend)
                    && public_instance_location(friend).is_none()
            })
            .collect::<Vec<_>>();
        let web = friends
            .iter()
            .copied()
            .filter(|friend| {
                !self.favorite_friend_ids.contains(&friend.user_id)
                    && friend.online
                    && !friend_is_in_vrchat(friend)
            })
            .collect::<Vec<_>>();
        let favorites = friends
            .iter()
            .copied()
            .filter(|friend| self.favorite_friend_ids.contains(&friend.user_id))
            .collect::<Vec<_>>();
        let offline = friends
            .into_iter()
            .filter(|friend| !friend.online && !self.favorite_friend_ids.contains(&friend.user_id))
            .collect::<Vec<_>>();

        let list = [
            (FriendSection::Favorites, favorites),
            (FriendSection::InVrchat, in_vrchat),
            (FriendSection::Private, private),
            (FriendSection::Web, web),
            (FriendSection::Offline, offline),
        ]
        .into_iter()
        .fold(column![].spacing(8), |column, (section, friends)| {
            let count = friends.len();
            let collapsed = self.collapsed_friend_sections.contains(&section);
            let rows: Element<'_, Message> = if collapsed {
                Space::new().height(0).into()
            } else {
                let visible = self.friend_section_limit(section).min(count);
                if section == FriendSection::Favorites {
                    self.favorite_sidebar_rows(friends, visible)
                } else if section == FriendSection::InVrchat {
                    self.public_instance_sidebar_rows(friends, visible)
                } else {
                    friends
                        .into_iter()
                        .take(visible)
                        .fold(column![].spacing(3), |rows, friend| {
                            rows.push(self.friend_sidebar_row(friend))
                        })
                        .into()
                }
            };
            column.push(
                column![
                    button(
                        row![
                            text(if collapsed { ">" } else { "v" }).size(11),
                            text(section.label()).size(11),
                            Space::new().width(Fill),
                            text(count.to_string()).size(10)
                        ]
                        .align_y(iced::Center)
                    )
                    .on_press(Message::ToggleFriendSection(section))
                    .style(button::text)
                    .padding([4, 2])
                    .width(Fill),
                    rule::horizontal(1),
                    rows
                ]
                .spacing(5),
            )
        });

        container(
            column![
                row![
                    text(t!("pages.friends").to_string()).size(16),
                    Space::new().width(Fill),
                    text(self.snapshot.friends.len().to_string()).size(11)
                ]
                .align_y(iced::Center),
                scrollable(list)
                    .spacing(SCROLLBAR_SPACING)
                    .height(Fill)
                    .on_scroll(|viewport| {
                        Message::FriendSidebarScrolled(viewport.relative_offset().y)
                    })
            ]
            .spacing(10),
        )
        .width(Length::Fixed(260.0))
        .height(Fill)
        .padding(12)
        .style(container::dark)
        .into()
    }

    fn favorite_sidebar_rows<'a>(
        &'a self,
        friends: Vec<&'a crate::store::FriendPresence>,
        visible: usize,
    ) -> Element<'a, Message> {
        let visible_ids = friends
            .iter()
            .take(visible)
            .map(|friend| friend.user_id.clone())
            .collect::<HashSet<_>>();
        let grouped_ids = self
            .favorite_friend_groups
            .iter()
            .flat_map(|group| group.friend_ids.iter().cloned())
            .collect::<HashSet<_>>();
        let mut rows = column![].spacing(5);
        for group in &self.favorite_friend_groups {
            let group_members = friends
                .iter()
                .copied()
                .filter(|friend| group.friend_ids.contains(&friend.user_id))
                .collect::<Vec<_>>();
            let count = group_members.len();
            let members = group_members
                .into_iter()
                .filter(|friend| visible_ids.contains(&friend.user_id))
                .collect::<Vec<_>>();
            rows = rows.push(
                row![
                    text(group.display_name.clone()).size(10),
                    Space::new().width(Fill),
                    text(count.to_string()).size(9)
                ]
                .padding([2, 4]),
            );
            rows = members.into_iter().fold(rows, |rows, friend| {
                rows.push(self.friend_sidebar_row(friend))
            });
        }
        let ungrouped_friends = friends
            .into_iter()
            .filter(|friend| !grouped_ids.contains(&friend.user_id))
            .collect::<Vec<_>>();
        let ungrouped_count = ungrouped_friends.len();
        let ungrouped = ungrouped_friends
            .into_iter()
            .filter(|friend| visible_ids.contains(&friend.user_id))
            .collect::<Vec<_>>();
        if !ungrouped.is_empty() {
            rows = rows.push(
                row![
                    text(t!("friends.ungrouped_favorites").to_string()).size(10),
                    Space::new().width(Fill),
                    text(ungrouped_count.to_string()).size(9)
                ]
                .padding([2, 4]),
            );
            rows = ungrouped.into_iter().fold(rows, |rows, friend| {
                rows.push(self.friend_sidebar_row(friend))
            });
        }
        rows.into()
    }

    fn public_instance_sidebar_rows<'a>(
        &'a self,
        friends: Vec<&'a crate::store::FriendPresence>,
        visible: usize,
    ) -> Element<'a, Message> {
        let visible_ids = friends
            .iter()
            .take(visible)
            .map(|friend| friend.user_id.clone())
            .collect::<HashSet<_>>();
        let mut instances = BTreeMap::<String, Vec<&crate::store::FriendPresence>>::new();
        for friend in friends {
            if let Some(location) = public_instance_location(friend) {
                instances
                    .entry(public_instance_key(location).to_string())
                    .or_default()
                    .push(friend);
            }
        }
        instances
            .into_iter()
            .fold(column![].spacing(5), |rows, (location, members)| {
                let count = members.len();
                let visible_members = members
                    .into_iter()
                    .filter(|friend| visible_ids.contains(&friend.user_id))
                    .collect::<Vec<_>>();
                let rows = rows.push(
                    row![
                        text(public_instance_label(&location, &self.world_names)).size(10),
                        Space::new().width(Fill),
                        text(count.to_string()).size(9)
                    ]
                    .padding([2, 4]),
                );
                visible_members.into_iter().fold(rows, |rows, friend| {
                    rows.push(self.friend_sidebar_row(friend))
                })
            })
            .into()
    }

    fn friend_sidebar_row<'a>(
        &'a self,
        friend: &'a crate::store::FriendPresence,
    ) -> Element<'a, Message> {
        let hover_id = format!("friend:{}", friend.user_id);
        let hovered = self.hovered_item.as_deref() == Some(hover_id.as_str());
        let status = friend_status(friend.status.as_deref(), friend.online);
        let location: Element<'a, Message> =
            if let Some(summary) = friend_secondary_text(friend, &self.world_names) {
                text(summary).size(10).into()
            } else {
                Space::new().height(0).into()
            };
        mouse_area(
            container(
                row![
                    self.friend_thumbnail(friend, 34.0),
                    status_dot(status),
                    column![
                        row![
                            text(friend.display_name.as_deref().unwrap_or(&friend.user_id))
                                .size(12),
                            if friend.traveling_to_location.is_some() {
                                text("🧳").size(12)
                            } else {
                                text("").size(12)
                            }
                        ]
                        .spacing(4),
                        location
                    ]
                    .width(Fill)
                    .spacing(2)
                ]
                .spacing(7)
                .align_y(iced::Center),
            )
            .padding([6, 4])
            .width(Fill)
            .style(rank_card_style(friend.trust_rank.clone(), hovered)),
        )
        .on_press(Message::OpenFriend(friend.user_id.clone()))
        .on_enter(Message::HoverItem(Some(hover_id)))
        .on_exit(Message::HoverItem(None))
        .interaction(mouse::Interaction::Pointer)
        .into()
    }

    fn topbar(&self) -> Element<'_, Message> {
        let search_placeholder = t!("search.global").to_string();
        let status = match (
            &self.snapshot.session.websocket_status,
            &self.snapshot.session.websocket_error,
        ) {
            (WebSocketStatus::Connected, _) => "LIVE".to_string(),
            (WebSocketStatus::Connecting, _) => "CONNECTING".to_string(),
            (WebSocketStatus::Disconnected, Some(error)) => {
                let reason: String = error.chars().take(100).collect();
                format!("OFFLINE: {reason}")
            }
            (WebSocketStatus::Disconnected, None) => "OFFLINE".to_string(),
        };

        column![
            row![
                column![
                    text(self.page.label()).size(24),
                    text(
                        self.session
                            .as_ref()
                            .map(|session| session.display_name.as_str())
                            .unwrap_or_default()
                    )
                    .size(13)
                ],
                Space::new().width(Fill),
                text(status).size(12),
                button(text(t!("actions.sync").to_string()).size(14))
                    .on_press(Message::Synchronize)
                    .style(button::secondary),
                button(text(t!("actions.refresh").to_string()).size(14))
                    .on_press(Message::Refresh)
                    .style(button::secondary)
            ]
            .align_y(iced::Center)
            .spacing(10),
            row![
                text_input(&search_placeholder, &self.search)
                    .on_input(Message::SearchChanged)
                    .on_submit(Message::SubmitSearch)
                    .padding(9)
                    .width(Fill)
            ]
            .align_y(iced::Center)
            .spacing(10)
        ]
        .spacing(10)
        .padding([12, 20])
        .into()
    }

    fn feedback(&self) -> Element<'_, Message> {
        if let Some(error) = &self.error {
            container(
                row![
                    text(error).size(13),
                    Space::new().width(Fill),
                    button(text(t!("actions.dismiss").to_string()).size(14))
                        .on_press(Message::ClearFeedback)
                        .style(button::text)
                ]
                .align_y(iced::Center),
            )
            .padding([8, 12])
            .style(container::bordered_box)
            .into()
        } else if let Some(notice) = &self.notice {
            container(
                row![
                    text(notice).size(13),
                    Space::new().width(Fill),
                    button(text(t!("actions.dismiss").to_string()).size(14))
                        .on_press(Message::ClearFeedback)
                        .style(button::text)
                ]
                .align_y(iced::Center),
            )
            .padding([8, 12])
            .into()
        } else {
            Space::new().height(0).into()
        }
    }

    fn page_view(&self) -> Element<'_, Message> {
        let content = match self.page {
            Page::Dashboard => self.dashboard(),
            Page::Friends => self.friends(),
            Page::Notifications => self.notifications(),
            Page::Api => self.api_workspace(),
            #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
            Page::VrOverlay => self.vr_overlay_settings(),
            _ => self.resource_list(),
        };

        container(content)
            .padding(20)
            .height(Fill)
            .width(Fill)
            .into()
    }

    #[cfg(all(feature = "vr-overlay", not(target_os = "macos")))]
    fn vr_overlay_settings(&self) -> Element<'_, Message> {
        let rows = column![
            text(t!("vr.title").to_string()).size(26),
            text(t!("vr.compiled_backends").to_string()).size(15),
            container(text(vr_build_summary()).size(14))
                .padding(14)
                .width(Fill)
                .style(container::bordered_box),
            text(t!("vr.build_help").to_string()).size(15),
            container(text(vr_build_command()).size(13))
                .padding(14)
                .width(Fill)
                .style(container::bordered_box),
            text(t!("vr.restart_hint").to_string()).size(12),
        ]
        .spacing(14);
        scrollable(rows).into()
    }

    fn dashboard(&self) -> Element<'_, Message> {
        let online = self
            .snapshot
            .friends
            .values()
            .filter(|friend| friend.online)
            .count();
        let stats = row![
            stat(
                t!("dashboard.online_friends").to_string(),
                online.to_string()
            ),
            stat(
                t!("dashboard.all_friends").to_string(),
                self.snapshot.friends.len().to_string()
            ),
            stat(
                t!("pages.notifications").to_string(),
                self.snapshot.notifications.len().to_string()
            ),
            stat(
                t!("dashboard.recent_events").to_string(),
                self.snapshot.recent_events.len().to_string()
            ),
        ]
        .spacing(12);

        column![
            stats,
            Space::new().height(18),
            text(t!("dashboard.recent_activity").to_string()).size(18),
            scrollable(self.snapshot.recent_events.iter().rev().take(30).fold(
                column![].spacing(6),
                |column, event| {
                    let (title, detail) = pipeline_event_summary(&event.event, &self.world_names);
                    column.push(
                        container(column![text(title).size(14), text(detail).size(12)].spacing(3))
                            .padding(10)
                            .width(Fill)
                            .style(container::bordered_box),
                    )
                }
            ))
            .spacing(SCROLLBAR_SPACING)
        ]
        .into()
    }

    fn friends(&self) -> Element<'_, Message> {
        let mut friends = self.snapshot.friends.values().collect::<Vec<_>>();
        friends.sort_by_key(|friend| {
            (
                !friend.online,
                friend
                    .display_name
                    .as_deref()
                    .unwrap_or(&friend.user_id)
                    .to_lowercase(),
            )
        });
        let mut sections = self
            .favorite_friend_groups
            .iter()
            .map(|group| {
                (
                    group.display_name.clone(),
                    friends
                        .iter()
                        .copied()
                        .filter(|friend| group.friend_ids.contains(&friend.user_id))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let grouped_favorite_ids = self
            .favorite_friend_groups
            .iter()
            .flat_map(|group| group.friend_ids.iter().cloned())
            .collect::<HashSet<_>>();
        let ungrouped_favorites = friends
            .iter()
            .copied()
            .filter(|friend| {
                self.favorite_friend_ids.contains(&friend.user_id)
                    && !grouped_favorite_ids.contains(&friend.user_id)
            })
            .collect::<Vec<_>>();
        if !ungrouped_favorites.is_empty() {
            sections.push((
                t!("friends.ungrouped_favorites").to_string(),
                ungrouped_favorites,
            ));
        }
        let mut public_instances = BTreeMap::<String, Vec<&crate::store::FriendPresence>>::new();
        for friend in friends
            .iter()
            .copied()
            .filter(|friend| !self.favorite_friend_ids.contains(&friend.user_id))
        {
            if let Some(location) = public_instance_location(friend) {
                public_instances
                    .entry(public_instance_key(location).to_string())
                    .or_default()
                    .push(friend);
            }
        }
        sections.extend(public_instances.into_iter().map(|(location, members)| {
            (public_instance_label(&location, &self.world_names), members)
        }));
        let private = friends
            .iter()
            .copied()
            .filter(|friend| {
                !self.favorite_friend_ids.contains(&friend.user_id)
                    && friend.online
                    && friend_is_in_vrchat(friend)
                    && public_instance_location(friend).is_none()
            })
            .collect::<Vec<_>>();
        let web = friends
            .iter()
            .copied()
            .filter(|friend| {
                !self.favorite_friend_ids.contains(&friend.user_id)
                    && friend.online
                    && !friend_is_in_vrchat(friend)
            })
            .collect::<Vec<_>>();
        let offline = friends
            .into_iter()
            .filter(|friend| !friend.online && !self.favorite_friend_ids.contains(&friend.user_id))
            .collect::<Vec<_>>();

        sections.extend([
            (t!("friends.sections.private").to_string(), private),
            (t!("friends.sections.web").to_string(), web),
            (t!("friends.offline").to_string(), offline),
        ]);

        let list = sections
            .into_iter()
            .fold(column![].spacing(8), |column, (label, friends)| {
                let count = friends.len();
                let rows = friends
                    .into_iter()
                    .fold(column![].spacing(6), |rows, friend| {
                        rows.push(self.friend_row(friend))
                    });
                column.push(
                    column![
                        row![
                            text(label).size(13),
                            text(count.to_string()).size(11),
                            rule::horizontal(1)
                        ]
                        .spacing(8)
                        .align_y(iced::Center),
                        rows
                    ]
                    .spacing(7),
                )
            });
        scrollable(list).spacing(SCROLLBAR_SPACING).into()
    }

    fn friend_row<'a>(&'a self, friend: &'a crate::store::FriendPresence) -> Element<'a, Message> {
        let hover_id = format!("friend:{}", friend.user_id);
        let hovered = self.hovered_item.as_deref() == Some(hover_id.as_str());
        let status = friend_status(friend.status.as_deref(), friend.online);
        let traveling = friend.traveling_to_location.is_some();
        let location: Element<'a, Message> =
            if let Some(summary) = friend_secondary_text(friend, &self.world_names) {
                text(summary).size(12).into()
            } else {
                Space::new().height(0).into()
            };
        mouse_area(
            container(
                row![
                    self.friend_thumbnail(friend, 48.0),
                    status_dot(status),
                    column![
                        row![
                            text(friend.display_name.as_deref().unwrap_or(&friend.user_id))
                                .size(14),
                            if traveling {
                                text("🧳").size(14)
                            } else {
                                text("").size(14)
                            }
                        ]
                        .spacing(6),
                        location
                    ]
                    .width(Fill),
                    button(text(t!("actions.remove").to_string()).size(14))
                        .on_press(Message::Unfriend(friend.user_id.clone()))
                        .style(button::danger)
                ]
                .align_y(iced::Center)
                .spacing(12),
            )
            .padding(10)
            .width(Fill)
            .style(rank_card_style(friend.trust_rank.clone(), hovered)),
        )
        .on_press(Message::OpenFriend(friend.user_id.clone()))
        .on_enter(Message::HoverItem(Some(hover_id)))
        .on_exit(Message::HoverItem(None))
        .interaction(mouse::Interaction::Pointer)
        .into()
    }

    fn notifications(&self) -> Element<'_, Message> {
        let list =
            self.snapshot
                .notifications
                .iter()
                .fold(column![].spacing(6), |column, (id, value)| {
                    let title = value
                        .get("message")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| t!("pages.notifications").to_string());
                    column.push(
                        container(
                            row![
                                column![text(title).size(14), text(id).size(11)].width(Fill),
                                button(text(t!("actions.delete").to_string()).size(14))
                                    .on_press(Message::DeleteNotification(id.clone()))
                                    .style(button::danger)
                            ]
                            .align_y(iced::Center),
                        )
                        .padding(10)
                        .width(Fill)
                        .style(container::bordered_box),
                    )
                });
        scrollable(list).spacing(SCROLLBAR_SPACING).into()
    }

    fn resource_list(&self) -> Element<'_, Message> {
        if self.page == Page::Favorites {
            return self.favorite_resource_list();
        }
        let list = self
            .results
            .iter()
            .enumerate()
            .take(self.resource_row_limit.min(self.results.len()))
            .fold(column![].spacing(6), |column, (index, item)| {
                column.push(self.resource_card(index, item))
            });

        column![
            if self.loading {
                text(t!("status.loading").to_string()).size(12)
            } else {
                text(t!("status.results", count = self.results.len()).to_string()).size(12)
            },
            scrollable(list)
                .spacing(SCROLLBAR_SPACING)
                .on_scroll(|viewport| { Message::ResourceScrolled(viewport.relative_offset().y) })
        ]
        .spacing(10)
        .into()
    }

    fn favorite_resource_list(&self) -> Element<'_, Message> {
        let mut groups = BTreeMap::<String, Vec<(usize, &ResultItem)>>::new();
        for (index, item) in self
            .results
            .iter()
            .enumerate()
            .take(self.resource_row_limit.min(self.results.len()))
        {
            for label in favorite_group_labels(&item.raw) {
                groups.entry(label).or_default().push((index, item));
            }
        }
        let list = groups
            .into_iter()
            .fold(column![].spacing(12), |column, (label, mut items)| {
                items.sort_by_key(|(_, item)| item.title.to_lowercase());
                let count = items.len();
                let rows = items
                    .into_iter()
                    .fold(column![].spacing(6), |rows, (index, item)| {
                        rows.push(self.resource_card(index, item))
                    });
                column.push(
                    column![
                        row![
                            text(label).size(14),
                            text(count.to_string()).size(11),
                            rule::horizontal(1)
                        ]
                        .spacing(8)
                        .align_y(iced::Center),
                        rows
                    ]
                    .spacing(7),
                )
            });
        column![
            if self.loading {
                text(t!("status.loading").to_string()).size(12)
            } else {
                text(t!("status.results", count = self.results.len()).to_string()).size(12)
            },
            scrollable(list)
                .spacing(SCROLLBAR_SPACING)
                .on_scroll(|viewport| Message::ResourceScrolled(viewport.relative_offset().y))
        ]
        .spacing(10)
        .into()
    }

    fn resource_card<'a>(&'a self, index: usize, item: &'a ResultItem) -> Element<'a, Message> {
        let hover_id = format!("resource:{}:{}", self.page.label(), item.id);
        let hovered = self.hovered_item.as_deref() == Some(hover_id.as_str());
        let badges = item.badges.iter().fold(row![].spacing(6), |row, badge| {
            row.push(
                container(text(badge).size(10))
                    .padding([3, 6])
                    .style(container::bordered_box),
            )
        });
        mouse_area(
            container(
                row![
                    self.thumbnail(item, 58.0),
                    if self.page == Page::Users {
                        status_dot(user_status_from_value(&item.raw))
                    } else {
                        Space::new()
                            .width(Length::Fixed(10.0))
                            .height(Length::Fixed(10.0))
                            .into()
                    },
                    column![
                        row![
                            text(&item.title).size(14),
                            if self.page == Page::Users && is_traveling(&item.raw) {
                                text("🧳").size(14)
                            } else {
                                text("").size(14)
                            },
                            Space::new().width(Fill),
                            text(&item.id).size(11)
                        ],
                        text(&item.subtitle).size(12),
                        badges
                    ]
                    .spacing(4)
                    .width(Fill)
                ]
                .spacing(12)
                .align_y(iced::Center),
            )
            .padding(10)
            .width(Fill)
            .style(item_card_style(item, hovered)),
        )
        .on_press(Message::OpenItem(index))
        .on_enter(Message::HoverItem(Some(hover_id)))
        .on_exit(Message::HoverItem(None))
        .interaction(mouse::Interaction::Pointer)
        .into()
    }

    fn thumbnail<'a>(&'a self, item: &'a ResultItem, size: f32) -> Element<'a, Message> {
        let width = if item.round_thumbnail {
            size
        } else {
            size * 1.45
        };
        let image_content: Element<'a, Message> = if let Some(handle) = item
            .thumbnail_url
            .as_ref()
            .and_then(|url| self.thumbnails.get(url))
        {
            image(handle.clone())
                .width(Length::Fixed(width))
                .height(Length::Fixed(size))
                .content_fit(ContentFit::Cover)
                .border_radius(if item.round_thumbnail {
                    size / 2.0
                } else {
                    6.0
                })
                .into()
        } else {
            container(text(item.title.chars().next().unwrap_or('?').to_string()).size(20))
                .width(Length::Fixed(width))
                .height(Length::Fixed(size))
                .center_x(Length::Fixed(width))
                .center_y(Length::Fixed(size))
                .style(container::bordered_box)
                .into()
        };

        let dots = platform_dots(&item.platforms, 7.0);
        stack![
            image_content,
            container(dots)
                .width(Length::Fixed(width))
                .height(Length::Fixed(size))
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom)
                .padding(4)
        ]
        .into()
    }

    fn friend_thumbnail<'a>(
        &'a self,
        friend: &'a crate::store::FriendPresence,
        size: f32,
    ) -> Element<'a, Message> {
        if let Some(handle) = friend
            .avatar_url
            .as_ref()
            .and_then(|url| self.thumbnails.get(url))
        {
            image(handle.clone())
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .content_fit(ContentFit::Cover)
                .border_radius(size / 2.0)
                .into()
        } else {
            container(
                text(
                    friend
                        .display_name
                        .as_deref()
                        .unwrap_or(&friend.user_id)
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_string(),
                )
                .size(18),
            )
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .center_x(Length::Fixed(size))
            .center_y(Length::Fixed(size))
            .style(container::bordered_box)
            .into()
        }
    }

    fn detail_modal(&self) -> Element<'_, Message> {
        let item = self.selected_item.as_ref().expect("selected item");
        let detail = self.selected_detail.as_ref().unwrap_or(&item.raw);
        let badges = resource_badges(detail)
            .into_iter()
            .fold(row![].spacing(6), |row, badge| {
                row.push(
                    container(text(badge).size(11))
                        .padding([4, 7])
                        .style(container::bordered_box),
                )
            });
        let fields = detail_fields(self.selected_page, detail).into_iter().fold(
            column![].spacing(8),
            |column, (label, value)| {
                column.push(
                    row![
                        text(label).size(11).width(Length::Fixed(130.0)),
                        text(value).size(13).width(Fill)
                    ]
                    .spacing(12),
                )
            },
        );
        let management: Element<'_, Message> = if let Some(favorite_id) = &item.favorite_id {
            button(text(t!("detail.remove_favorite").to_string()).size(14))
                .on_press(Message::RemoveFavorite(favorite_id.clone()))
                .style(button::danger)
                .into()
        } else {
            match self.selected_page {
                Page::Users | Page::Friends
                    if json_bool(detail, &["isFriend", "social.isFriend"])
                        || self.selected_page == Page::Friends =>
                {
                    button(text(t!("detail.remove_friend").to_string()).size(14))
                        .on_press(Message::Unfriend(item.id.clone()))
                        .style(button::danger)
                        .into()
                }
                Page::Groups
                    if json_string(detail, &["membershipStatus"])
                        .is_some_and(|status| matches!(status.as_str(), "member" | "joined")) =>
                {
                    button(text(t!("detail.leave_group").to_string()).size(14))
                        .on_press(Message::LeaveGroup(item.id.clone()))
                        .style(button::danger)
                        .into()
                }
                _ => {
                    if let Some(label) = resource_action(self.selected_page) {
                        button(text(label).size(14))
                            .on_press(Message::ResourceAction(self.selected_page, item.id.clone()))
                            .style(if matches!(self.selected_page, Page::Files | Page::Props) {
                                button::danger
                            } else {
                                button::primary
                            })
                            .into()
                    } else {
                        Space::new().width(0).into()
                    }
                }
            }
        };
        let is_user = matches!(self.selected_page, Page::Users | Page::Friends);
        let tabs: Element<'_, Message> = DetailTab::ALL
            .into_iter()
            .fold(row![].spacing(6), |row, tab| {
                row.push(
                    button(text(tab.label()).size(14))
                        .on_press(Message::DetailTabSelected(tab))
                        .style(if self.detail_tab == tab {
                            button::primary
                        } else {
                            button::secondary
                        }),
                )
            })
            .into();
        let body: Element<'_, Message> = if !is_user || self.detail_tab == DetailTab::Overview {
            scrollable(fields)
                .spacing(SCROLLBAR_SPACING)
                .height(Length::Fixed(360.0))
                .into()
        } else if self.relations_loading {
            container(text(t!("detail.loading_related").to_string()).size(12))
                .height(Length::Fixed(360.0))
                .into()
        } else {
            let (items, page) = match self.detail_tab {
                DetailTab::MutualFriends => (&self.user_relations.mutual_friends, Page::Users),
                DetailTab::MutualGroups => (&self.user_relations.mutual_groups, Page::Groups),
                DetailTab::Groups => (&self.user_relations.groups, Page::Groups),
                DetailTab::FriendGroups => (&self.user_relations.friend_groups, Page::Favorites),
                DetailTab::Overview => unreachable!(),
            };
            self.relation_list(items, page)
        };
        let detail_heading: Element<'_, Message> = if self.detail_loading {
            text(t!("detail.loading_details").to_string())
                .size(12)
                .into()
        } else {
            text(self.detail_tab.label()).size(16).into()
        };

        container(
            column![
                row![
                    self.thumbnail(item, 96.0),
                    column![text(&item.title).size(24), text(&item.id).size(11), badges]
                        .spacing(7)
                        .width(Fill),
                    button(text(t!("actions.close").to_string()).size(14))
                        .on_press(Message::CloseItem)
                        .style(button::secondary)
                ]
                .spacing(14)
                .align_y(iced::Center),
                rule::horizontal(1),
                if is_user {
                    tabs
                } else {
                    Space::new().height(0).into()
                },
                detail_heading,
                body,
                rule::horizontal(1),
                row![
                    text(t!("detail.management").to_string()).size(14),
                    Space::new().width(Fill),
                    management
                ]
                .align_y(iced::Center)
            ]
            .spacing(14),
        )
        .width(Length::Fixed(720.0))
        .max_width(720.0)
        .padding(18)
        .style(container::bordered_box)
        .into()
    }

    fn relation_list<'a>(&'a self, items: &'a [ResultItem], page: Page) -> Element<'a, Message> {
        if items.is_empty() {
            return container(text(t!("status.no_items").to_string()).size(13))
                .height(Length::Fixed(360.0))
                .center_y(Length::Fixed(360.0))
                .into();
        }
        if page == Page::Favorites {
            let mut remaining_members = self.relation_row_limit;
            let groups = items.iter().fold(column![].spacing(10), |column, group| {
                if remaining_members == 0 {
                    return column;
                }
                let visible_members = remaining_members;
                let mut rendered_members = 0usize;
                let members = group
                    .raw
                    .get("members")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .take(visible_members)
                    .cloned()
                    .map(result_item)
                    .fold(column![].spacing(4), |members, member| {
                        rendered_members += 1;
                        let hover_id = format!("related:user:{}", member.id);
                        let hovered = self.hovered_item.as_deref() == Some(hover_id.as_str());
                        let thumbnail: Element<'_, Message> = member
                            .thumbnail_url
                            .as_ref()
                            .and_then(|url| self.thumbnails.get(url))
                            .map(|handle| {
                                image(handle.clone())
                                    .width(Length::Fixed(34.0))
                                    .height(Length::Fixed(34.0))
                                    .content_fit(ContentFit::Cover)
                                    .border_radius(17.0)
                                    .into()
                            })
                            .unwrap_or_else(|| {
                                container(
                                    text(member.title.chars().next().unwrap_or('?').to_string())
                                        .size(13),
                                )
                                .width(Length::Fixed(34.0))
                                .height(Length::Fixed(34.0))
                                .center_x(Length::Fixed(34.0))
                                .center_y(Length::Fixed(34.0))
                                .style(container::bordered_box)
                                .into()
                            });
                        let title = member.title.clone();
                        members.push(
                            mouse_area(
                                container(
                                    row![
                                        thumbnail,
                                        text(title).size(12).width(Fill),
                                        text(">").size(14)
                                    ]
                                    .spacing(8)
                                    .align_y(iced::Center),
                                )
                                .padding([6, 8])
                                .width(Fill)
                                .style(related_card_style(hovered)),
                            )
                            .on_press(Message::OpenRelated(member, Page::Users))
                            .on_enter(Message::HoverItem(Some(hover_id)))
                            .on_exit(Message::HoverItem(None))
                            .interaction(mouse::Interaction::Pointer),
                        )
                    });
                remaining_members = remaining_members.saturating_sub(rendered_members);
                column.push(
                    column![
                        row![
                            text(&group.title).size(14).width(Fill),
                            text(&group.subtitle).size(11)
                        ],
                        members
                    ]
                    .spacing(6),
                )
            });
            return scrollable(groups)
                .spacing(SCROLLBAR_SPACING)
                .height(Length::Fixed(360.0))
                .on_scroll(|viewport| Message::RelationScrolled(viewport.relative_offset().y))
                .into();
        }
        let rows = items
            .iter()
            .take(self.relation_row_limit.min(items.len()))
            .fold(column![].spacing(6), |column, item| {
                let hover_id = format!("related:{}:{}", page.label(), item.id);
                let hovered = self.hovered_item.as_deref() == Some(hover_id.as_str());
                column.push(
                    mouse_area(
                        container(
                            row![
                                self.thumbnail(item, 44.0),
                                column![text(&item.title).size(13), text(&item.subtitle).size(11)]
                                    .spacing(3)
                                    .width(Fill),
                                text(">").size(14)
                            ]
                            .spacing(10)
                            .align_y(iced::Center),
                        )
                        .padding(8)
                        .width(Fill)
                        .style(related_card_style(hovered)),
                    )
                    .on_press(Message::OpenRelated(item.clone(), page))
                    .on_enter(Message::HoverItem(Some(hover_id)))
                    .on_exit(Message::HoverItem(None))
                    .interaction(mouse::Interaction::Pointer),
                )
            });
        scrollable(rows)
            .spacing(SCROLLBAR_SPACING)
            .height(Length::Fixed(360.0))
            .on_scroll(|viewport| Message::RelationScrolled(viewport.relative_offset().y))
            .into()
    }

    fn api_workspace(&self) -> Element<'_, Message> {
        let route_placeholder = t!("api.route_placeholder").to_string();
        let body_placeholder = t!("api.body_placeholder").to_string();
        let methods = ["GET", "POST", "PUT", "PATCH", "DELETE"].into_iter().fold(
            row![].spacing(6),
            |row, method| {
                row.push(
                    button(method)
                        .on_press(Message::ApiMethodChanged(method.to_string()))
                        .style(if self.api_method == method {
                            button::primary
                        } else {
                            button::secondary
                        }),
                )
            },
        );

        column![
            text(t!("api.title").to_string()).size(18),
            methods,
            text_input(&route_placeholder, &self.api_path)
                .on_input(Message::ApiPathChanged)
                .padding(10),
            text_input(&body_placeholder, &self.api_body)
                .on_input(Message::ApiBodyChanged)
                .padding(10),
            button(text(t!("actions.execute").to_string()).size(14))
                .on_press_maybe((!self.loading).then_some(Message::ExecuteApi))
                .style(button::primary),
            text(t!("api.response").to_string()).size(14),
            scrollable(
                container(text(&self.api_response).size(12))
                    .padding(12)
                    .width(Fill)
                    .style(container::bordered_box)
            )
            .spacing(SCROLLBAR_SPACING)
        ]
        .spacing(10)
        .into()
    }
}

fn app_theme(_: &App, _: window::Id) -> Theme {
    Theme::TokyoNight
}

fn app_title(_: &App, _: window::Id) -> String {
    "VRCX - BIR".to_string()
}

fn app_style(_: &App, theme: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}

fn snapshot_stream(
    subscription: &SnapshotSubscription,
) -> impl futures_util::Stream<Item = Message> + use<> {
    stream::unfold(
        (subscription.receiver.clone(), false),
        |(mut receiver, initialized)| async move {
            if initialized {
                receiver.changed().await.ok()?;
            }
            let snapshot = receiver.borrow_and_update().clone();
            Some((Message::SnapshotLoaded(snapshot), (receiver, true)))
        },
    )
}

async fn load_favorite_resources(
    backend: Arc<Backend>,
    tag: Option<String>,
) -> Result<Vec<Value>, String> {
    let groups = backend
        .api()
        .favorite_groups()
        .await
        .map_err(|error| error.to_string())?
        .groups;
    let mut favorites = Vec::new();
    let mut offset = 0;
    loop {
        let page = backend
            .api()
            .favorites(&FavoritesQuery {
                page: PaginationQuery::new().limit(100).offset(offset),
                r#type: None,
                tag: tag.clone(),
            })
            .await
            .map_err(|error| error.to_string())?;
        let count = page.items.len();
        favorites.extend(page.items);
        if count < 100 {
            break;
        }
        offset += 100;
    }

    Ok(stream::iter(favorites)
        .map(|favorite| {
            let backend = backend.clone();
            let groups = groups.clone();
            async move {
                let resource_path = match &favorite.favorite_type {
                    crate::models::favorite::FavoriteType::Friend => {
                        format!("users/{}", favorite.favorite_id)
                    }
                    crate::models::favorite::FavoriteType::World => {
                        format!("worlds/{}", favorite.favorite_id)
                    }
                    crate::models::favorite::FavoriteType::Avatar => {
                        format!("avatars/{}", favorite.favorite_id)
                    }
                    crate::models::favorite::FavoriteType::Prop => {
                        format!("props/{}", favorite.favorite_id)
                    }
                    crate::models::favorite::FavoriteType::Group => {
                        format!("groups/{}", favorite.favorite_id)
                    }
                    crate::models::favorite::FavoriteType::Unknown => String::new(),
                };
                let mut resource = if resource_path.is_empty() {
                    Value::Null
                } else {
                    backend
                        .api()
                        .request_value(reqwest::Method::GET, &resource_path, None)
                        .await
                        .unwrap_or_else(|error| {
                            tracing::warn!(
                                favorite_id = favorite.id,
                                resource_id = favorite.favorite_id,
                                %error,
                                "favorite resource hydration failed"
                            );
                            Value::Null
                        })
                };
                if !resource.is_object() {
                    resource = serde_json::json!({
                        "id": favorite.favorite_id,
                        "name": favorite.favorite_id,
                        "description": "Favorite resource unavailable",
                    });
                }
                if let Some(object) = resource.as_object_mut() {
                    object.insert(
                        "_favoriteId".to_string(),
                        Value::String(favorite.id.clone()),
                    );
                    object.insert(
                        "_favoriteType".to_string(),
                        serde_json::to_value(favorite.favorite_type).unwrap_or(Value::Null),
                    );
                    object.insert(
                        "_favoriteTags".to_string(),
                        serde_json::to_value(&favorite.tags).unwrap_or(Value::Null),
                    );
                    let favorite_groups = favorite_display_groups(&favorite, &groups);
                    object.insert(
                        "_favoriteGroups".to_string(),
                        serde_json::to_value(favorite_groups).unwrap_or(Value::Null),
                    );
                }
                resource
            }
        })
        .buffer_unordered(10)
        .collect()
        .await)
}

async fn load_user_relations(
    backend: Arc<Backend>,
    user_id: String,
) -> Result<UserRelations, String> {
    let api = backend.api();
    let friend_query = FavoritesQuery {
        page: PaginationQuery::new().limit(100),
        r#type: Some(crate::models::favorite::FavoriteType::Friend),
        tag: None,
    };
    let (mutual_friends, mutual_groups, groups, favorite_groups, favorites, snapshot) = tokio::join!(
        api.mutual_friends(&user_id),
        api.mutual_groups(&user_id),
        api.user_groups_raw(&user_id),
        api.favorite_groups(),
        api.favorites(&friend_query),
        backend.store().snapshot(),
    );

    let values = |result: Result<Vec<Value>, crate::error::VrcError>, label: &str| {
        result.unwrap_or_else(|error| {
            tracing::warn!(user_id, collection = label, %error, "related user data unavailable");
            Vec::new()
        })
    };
    let mutual_friends = values(mutual_friends, "mutual friends")
        .into_iter()
        .map(result_item)
        .collect();
    let mutual_groups = values(mutual_groups, "mutual groups")
        .into_iter()
        .map(normalize_group_value)
        .map(result_item)
        .collect();
    let groups = values(groups, "groups")
        .into_iter()
        .map(normalize_group_value)
        .map(result_item)
        .collect();

    let favorite_groups = favorite_groups
        .map(|groups| groups.groups)
        .unwrap_or_else(|error| {
            tracing::warn!(user_id, %error, "friend favorite groups unavailable");
            Vec::new()
        });
    let favorites = favorites.map(|list| list.items).unwrap_or_else(|error| {
        tracing::warn!(user_id, %error, "friend favorites unavailable");
        Vec::new()
    });
    let friend_groups = favorite_groups
        .into_iter()
        .filter(|group| {
            matches!(
                group.favorite_type,
                crate::models::favorite::FavoriteType::Friend
            )
        })
        .map(|group| {
            let members = favorites
                .iter()
                .filter(|favorite| {
                    favorite.tags.iter().any(|tag| {
                        tag == &group.name || group.tags.iter().any(|group_tag| group_tag == tag)
                    })
                })
                .map(|favorite| {
                    let presence = snapshot.friends.get(&favorite.favorite_id);
                    serde_json::json!({
                        "id": favorite.favorite_id,
                        "displayName": presence.and_then(|friend| friend.display_name.clone())
                            .unwrap_or_else(|| favorite.favorite_id.clone()),
                        "userIcon": presence.and_then(|friend| friend.avatar_url.clone()),
                    })
                })
                .collect::<Vec<_>>();
            result_item(serde_json::json!({
                "id": group.id,
                "name": group.display_name,
                "description": format!("{} friends", members.len()),
                "favoriteGroup": group.name,
                "members": members,
            }))
        })
        .collect();

    Ok(UserRelations {
        mutual_friends,
        mutual_groups,
        groups,
        friend_groups,
    })
}

fn normalize_group_value(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        if !object.contains_key("id") {
            if let Some(group_id) = object.get("groupId").cloned() {
                object.insert("id".to_string(), group_id);
            }
        }
    }
    value
}

fn favorite_group_labels(value: &Value) -> Vec<String> {
    let kind = match value.get("_favoriteType").and_then(Value::as_str) {
        Some("friend") => t!("pages.friends").to_string(),
        Some("world") => t!("pages.worlds").to_string(),
        Some("avatar") => t!("pages.avatars").to_string(),
        _ => t!("pages.favorites").to_string(),
    };
    let labels = value
        .get("_favoriteGroups")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|label| !label.trim().is_empty())
        .map(|label| format!("{kind} · {label}"))
        .collect::<Vec<_>>();
    if !labels.is_empty() {
        return labels;
    }
    vec![t!("favorites.fallback_group", kind = kind).to_string()]
}

fn favorite_group_matches_tag(group: &crate::models::favorite::FavoriteGroup, tag: &str) -> bool {
    group.name == tag || group.tags.iter().any(|group_tag| group_tag == tag)
}

fn favorite_display_groups(
    favorite: &crate::models::favorite::Favorite,
    groups: &[crate::models::favorite::FavoriteGroup],
) -> Vec<String> {
    let mut seen_labels = HashSet::new();
    favorite
        .tags
        .iter()
        .filter(|tag| !tag.trim().is_empty())
        .filter_map(|tag| {
            let label = groups
                .iter()
                .find(|group| {
                    group.favorite_type == favorite.favorite_type
                        && favorite_group_matches_tag(group, tag)
                })
                .map(|group| {
                    if group.display_name.trim().is_empty() {
                        group.name.clone()
                    } else {
                        group.display_name.clone()
                    }
                })
                .unwrap_or_else(|| tag.clone());
            seen_labels.insert(label.clone()).then_some(label)
        })
        .collect()
}

fn pipeline_event_summary(
    event: &PipelineEvent,
    world_names: &HashMap<String, String>,
) -> (&'static str, String) {
    match event {
        PipelineEvent::FriendAdd(content) => {
            ("Friend added", user_label(&content.user, &content.user_id))
        }
        PipelineEvent::FriendDelete { user_id } => ("Friend removed", user_id.clone()),
        PipelineEvent::FriendOnline(content) => (
            "Friend online",
            format!(
                "{} on {}",
                user_label(&content.user, &content.user_id),
                content.platform
            ),
        ),
        PipelineEvent::FriendActive(content) => (
            "Friend active",
            format!(
                "{} on {}",
                user_label(&content.user, &content.user_id),
                content.platform
            ),
        ),
        PipelineEvent::FriendOffline(content) => (
            "Friend offline",
            format!("{} on {}", content.user_id, content.platform),
        ),
        PipelineEvent::FriendUpdate(content) => (
            "Friend updated",
            user_label(&content.user, &content.user_id),
        ),
        PipelineEvent::FriendLocation(content) => (
            "Friend location changed",
            format!(
                "{} - {}",
                user_label(&content.user, &content.user_id),
                location_label(&content.location, world_names)
            ),
        ),
        PipelineEvent::UserUpdate(content) => {
            ("User updated", user_label(&content.user, &content.user_id))
        }
        PipelineEvent::UserLocation(content) => (
            "Your location changed",
            location_label(&content.location, world_names),
        ),
        PipelineEvent::Notification(value) | PipelineEvent::NotificationV2(value) => (
            "Notification received",
            value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("New notification")
                .chars()
                .take(120)
                .collect(),
        ),
        PipelineEvent::ResponseNotification(_) => {
            ("Notification answered", "A response was sent".to_string())
        }
        PipelineEvent::SeeNotification(_) => ("Notification seen", "Marked as seen".to_string()),
        PipelineEvent::HideNotification(_) => (
            "Notification hidden",
            "Removed from the active list".to_string(),
        ),
        PipelineEvent::ClearNotification => (
            "Notifications cleared",
            "All notifications were cleared".to_string(),
        ),
        PipelineEvent::NotificationV2Update(_) => (
            "Notification updated",
            "Notification content changed".to_string(),
        ),
        PipelineEvent::NotificationV2Delete(deleted) => (
            "Notification deleted",
            format!("{} item(s)", deleted.ids.len()),
        ),
        PipelineEvent::UserBadgeAssigned(_) => ("Badge assigned", "A badge was added".to_string()),
        PipelineEvent::UserBadgeUnassigned(_) => {
            ("Badge removed", "A badge was removed".to_string())
        }
        PipelineEvent::ContentRefresh(content) => (
            "Content refreshed",
            format!("{} - {}", content.content_type, content.action_type),
        ),
        PipelineEvent::EconomyUpdate(_) => ("Economy updated", "Purchase data changed".to_string()),
        PipelineEvent::ModifiedImageUpdate(_) => (
            "Image updated",
            "Image processing status changed".to_string(),
        ),
        PipelineEvent::InstanceQueueJoined(content) => (
            "Instance queue joined",
            format!("Position {}", content.position),
        ),
        PipelineEvent::InstanceQueueReady(_) => (
            "Instance ready",
            "The queued instance is available".to_string(),
        ),
        PipelineEvent::GroupJoined(content) => ("Group joined", content.group_id.clone()),
        PipelineEvent::GroupLeft(content) => ("Group left", content.group_id.clone()),
        PipelineEvent::GroupMemberUpdated(_) => (
            "Group member updated",
            "Membership data changed".to_string(),
        ),
        PipelineEvent::GroupRoleUpdated(_) => {
            ("Group role updated", "Role data changed".to_string())
        }
        PipelineEvent::Unknown { event_type, .. } => ("Pipeline event", event_type.clone()),
    }
}

fn user_label(user: &Value, fallback: &str) -> String {
    user.get("displayName")
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

fn location_label(location: &str, world_names: &HashMap<String, String>) -> String {
    if location == "private" || location.contains("private") {
        "Private instance".to_string()
    } else if location == "offline" || location.is_empty() {
        "Offline".to_string()
    } else if let Some(world_id) = world_id_from_location(location) {
        world_names
            .get(world_id)
            .cloned()
            .unwrap_or_else(|| world_id.to_string())
    } else {
        location.to_string()
    }
}

fn friend_location(
    friend: &crate::store::FriendPresence,
    world_names: &HashMap<String, String>,
) -> Option<String> {
    friend
        .traveling_to_location
        .as_deref()
        .or(friend.location.as_deref())
        .filter(|location| !location.is_empty() && *location != "offline")
        .map(|location| location_label(location, world_names))
}

fn friend_secondary_text(
    friend: &crate::store::FriendPresence,
    world_names: &HashMap<String, String>,
) -> Option<String> {
    friend
        .status_description
        .as_deref()
        .map(str::trim)
        .filter(|status| !status.is_empty())
        .map(str::to_string)
        .or_else(|| friend_location(friend, world_names))
}

fn friend_instance_location(friend: &crate::store::FriendPresence) -> Option<&str> {
    friend
        .traveling_to_location
        .as_deref()
        .or(friend.location.as_deref())
        .filter(|location| !location.is_empty() && *location != "offline")
}

fn public_instance_location(friend: &crate::store::FriendPresence) -> Option<&str> {
    if !friend.online {
        return None;
    }
    let location = friend_instance_location(friend)?;
    if !location.starts_with("wrld_")
        || [
            "~private(",
            "~friends(",
            "~hidden(",
            "~invite(",
            "~invite+(",
            "~group(",
        ]
        .iter()
        .any(|marker| location.contains(marker))
    {
        return None;
    }
    Some(location)
}

fn public_instance_label(location: &str, world_names: &HashMap<String, String>) -> String {
    let world_id = world_id_from_location(location).unwrap_or(location);
    let world = world_names
        .get(world_id)
        .map(String::as_str)
        .unwrap_or(world_id);
    let instance = location
        .split_once(':')
        .map(|(_, instance)| instance.split('~').next().unwrap_or(instance))
        .filter(|instance| !instance.is_empty());
    instance.map_or_else(
        || world.to_string(),
        |instance| format!("{world} · {instance}"),
    )
}

fn public_instance_key(location: &str) -> &str {
    location.split('~').next().unwrap_or(location)
}

fn friend_is_in_vrchat(friend: &crate::store::FriendPresence) -> bool {
    friend
        .traveling_to_location
        .as_deref()
        .or(friend.location.as_deref())
        .is_some_and(|location| !location.is_empty() && location != "offline")
}

fn friend_section(
    friend: &crate::store::FriendPresence,
    favorite_friend_ids: &HashSet<String>,
) -> FriendSection {
    if favorite_friend_ids.contains(&friend.user_id) {
        FriendSection::Favorites
    } else if public_instance_location(friend).is_some() {
        FriendSection::InVrchat
    } else if friend_is_in_vrchat(friend) {
        FriendSection::Private
    } else if friend.online {
        FriendSection::Web
    } else {
        FriendSection::Offline
    }
}

fn world_id_from_location(location: &str) -> Option<&str> {
    let world_id = location.split(':').next().unwrap_or(location);
    world_id.starts_with("wrld_").then_some(world_id)
}

fn event_world_ids(event: &PipelineEvent) -> Vec<&str> {
    match event {
        PipelineEvent::FriendOnline(content) => world_id_from_location(&content.location)
            .into_iter()
            .collect(),
        PipelineEvent::FriendLocation(content) => [
            world_id_from_location(&content.location),
            world_id_from_location(&content.traveling_to_location),
            world_id_from_location(&content.world_id),
        ]
        .into_iter()
        .flatten()
        .collect(),
        PipelineEvent::UserLocation(content) => [
            world_id_from_location(&content.location),
            world_id_from_location(&content.traveling_to_location),
        ]
        .into_iter()
        .flatten()
        .collect(),
        _ => Vec::new(),
    }
}

fn stat(label: String, value: String) -> Element<'static, Message> {
    container(column![text(label).size(12), text(value).size(28)].spacing(4))
        .padding(14)
        .width(Fill)
        .style(container::bordered_box)
        .into()
}

fn account_avatar<'a>(
    name: &str,
    avatar_url: Option<&str>,
    thumbnails: &'a HashMap<String, image::Handle>,
) -> Element<'a, Message> {
    if let Some(handle) = avatar_url.and_then(|url| thumbnails.get(url)) {
        return image(handle.clone())
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(30.0))
            .content_fit(ContentFit::Cover)
            .border_radius(15.0)
            .into();
    }

    let initials = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();
    let initials = if initials.is_empty() {
        "VR".to_string()
    } else {
        initials.to_uppercase()
    };

    container(text(initials).size(10))
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .center_x(Length::Fixed(30.0))
        .center_y(Length::Fixed(30.0))
        .style(|_| {
            container::Style::default()
                .background(Color::from_rgb8(219, 145, 29))
                .border(Border {
                    width: 0.0,
                    radius: 15.0.into(),
                    color: Color::from_rgb8(219, 145, 29),
                })
        })
        .into()
}

fn account_menu_button(
    icon: &'static str,
    label: String,
    message: Message,
) -> Element<'static, Message> {
    button(
        row![text(icon).size(15), text(label).size(14).width(Fill)]
            .spacing(10)
            .align_y(iced::Center),
    )
    .on_press(message)
    .style(button::text)
    .padding([7, 4])
    .width(Fill)
    .into()
}

fn result_item(raw: Value) -> ResultItem {
    let id = string_field(&raw, &["id", "identifier.id", "user.id"]);
    let title = string_field(
        &raw,
        &[
            "displayName",
            "name",
            "title",
            "identifier.name",
            "profile.name",
        ],
    );
    let subtitle = string_field(
        &raw,
        &[
            "description",
            "statusDescription",
            "authorName",
            "itemTypeLabel",
        ],
    );
    let thumbnail_url = non_empty(string_field(
        &raw,
        &[
            "userIcon",
            "profilePicOverrideThumbnail",
            "currentAvatarThumbnailImageUrl",
            "currentAvatarImageUrl",
            "profile.userIcon",
            "profile.profilePicOverrideThumbnail",
            "profile.currentAvatarThumbnailImageUrl",
            "profile.currentAvatarImageUrl",
            "thumbnailImageUrl",
            "iconUrl",
            "bannerUrl",
            "imageUrl",
            "media.thumbnailImageUrl",
            "media.images.thumbnailImageUrl",
        ],
    ));
    let round_thumbnail = resource_is_round(&raw);
    let platforms = content_platforms(&raw);
    let badges = resource_badges(&raw);
    let trust_rank = resource_trust_rank(&raw);
    let favorite_id = non_empty(string_field(&raw, &["_favoriteId"]));
    let compact_raw = compact_result_raw(&raw);
    ResultItem {
        id,
        title: if title.is_empty() {
            "Untitled".to_string()
        } else {
            title
        },
        subtitle,
        thumbnail_url,
        round_thumbnail,
        platforms,
        badges,
        trust_rank,
        favorite_id,
        raw: compact_raw,
    }
}

fn compact_result_raw(raw: &Value) -> Value {
    let Some(object) = raw.as_object() else {
        return raw.clone();
    };
    let keep = [
        "id",
        "displayName",
        "name",
        "title",
        "description",
        "status",
        "statusDescription",
        "authorName",
        "itemTypeLabel",
        "membershipStatus",
        "releaseStatus",
        "isFriend",
        "isVerified",
        "ageVerified",
        "ageVerificationStatus",
        "travelingToLocation",
        "location",
        "last_platform",
        "performance",
        "tags",
        "_favoriteId",
        "_favoriteType",
        "_favoriteTags",
        "members",
    ];
    let mut compact = serde_json::Map::new();
    for key in keep {
        if let Some(value) = object.get(key) {
            compact.insert(key.to_string(), value.clone());
        }
    }
    for key in [
        "identifier",
        "identity",
        "profile",
        "presence",
        "social",
        "capacity",
        "stats",
        "publication",
        "publications",
        "metadata",
    ] {
        if let Some(value) = object.get(key) {
            compact.insert(key.to_string(), value.clone());
        }
    }
    Value::Object(compact)
}

fn friend_result_item(friend: &crate::store::FriendPresence) -> ResultItem {
    let raw = serde_json::json!({
        "id": friend.user_id,
        "displayName": friend.display_name,
        "status": friend.status,
        "statusDescription": friend.status_description,
        "isFriend": true,
        "location": friend.location,
        "travelingToLocation": friend.traveling_to_location,
        "trustRank": friend.trust_rank,
    });
    ResultItem {
        id: friend.user_id.clone(),
        title: friend
            .display_name
            .clone()
            .unwrap_or_else(|| friend.user_id.clone()),
        subtitle: friend_secondary_text(friend, &HashMap::new())
            .unwrap_or_else(|| "Offline".to_string()),
        thumbnail_url: friend.avatar_url.clone(),
        round_thumbnail: true,
        platforms: Vec::new(),
        badges: resource_badges(&raw),
        trust_rank: friend.trust_rank.clone(),
        favorite_id: None,
        raw,
    }
}

fn string_field(value: &Value, paths: &[&str]) -> String {
    paths
        .iter()
        .find_map(|path| {
            path.split('.')
                .try_fold(value, |value, key| value.get(key))
                .and_then(Value::as_str)
        })
        .unwrap_or_default()
        .to_string()
}

fn compact_json(value: &Value) -> String {
    let text = serde_json::to_string(value).unwrap_or_default();
    let mut chars = text.chars();
    let compact = chars.by_ref().take(237).collect::<String>();
    if chars.next().is_some() {
        format!("{compact}...")
    } else {
        compact
    }
}

fn resource_is_round(value: &Value) -> bool {
    string_field(value, &["id", "identifier.id"])
        .split('_')
        .next()
        .is_some_and(|prefix| matches!(prefix, "usr" | "avtr"))
}

fn content_platforms(value: &Value) -> Vec<ContentPlatform> {
    let mut platforms = Vec::new();
    for platform in value
        .get("unityPackages")
        .or_else(|| {
            value
                .get("content")
                .and_then(|content| content.get("unityPackage"))
        })
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|package| package.get("platform").and_then(Value::as_str))
    {
        push_platform(&mut platforms, platform);
    }

    if let Some(performance) = value.get("performance").and_then(Value::as_object) {
        for key in performance.keys() {
            push_platform(&mut platforms, key);
        }
    }

    platforms
}

fn push_platform(platforms: &mut Vec<ContentPlatform>, platform: &str) {
    let platform = match platform.to_ascii_lowercase().as_str() {
        "standalonewindows" | "windows" | "win" => ContentPlatform::Windows,
        "android" | "quest" => ContentPlatform::Android,
        "ios" => ContentPlatform::Ios,
        _ => return,
    };
    if !platforms.contains(&platform) {
        platforms.push(platform);
    }
}

fn platform_dots(platforms: &[ContentPlatform], size: f32) -> Element<'_, Message> {
    platforms
        .iter()
        .copied()
        .fold(row![].spacing(3), |row, platform| {
            let color = match platform {
                ContentPlatform::Windows => Color::from_rgb8(64, 156, 255),
                ContentPlatform::Android => Color::from_rgb8(52, 199, 89),
                ContentPlatform::Ios => Color::from_rgb8(142, 142, 147),
            };
            row.push(
                container(Space::new())
                    .width(Length::Fixed(size))
                    .height(Length::Fixed(size))
                    .style(move |_| {
                        container::Style::default()
                            .background(color)
                            .border(Border {
                                width: 1.0,
                                radius: (size / 2.0).into(),
                                color: Color::from_rgb8(18, 20, 28),
                            })
                    }),
            )
        })
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresenceStatus {
    Online,
    JoinMe,
    AskMe,
    DoNotDisturb,
    Offline,
}

fn friend_status(status: Option<&str>, online: bool) -> PresenceStatus {
    if !online {
        return PresenceStatus::Offline;
    }
    match status {
        Some("joinMe") => PresenceStatus::JoinMe,
        Some("askMe") => PresenceStatus::AskMe,
        Some("busy") => PresenceStatus::DoNotDisturb,
        Some("offline") => PresenceStatus::Offline,
        _ => PresenceStatus::Online,
    }
}

fn user_status_from_value(value: &Value) -> PresenceStatus {
    let status = json_string(value, &["status", "identity.status.status"]);
    status
        .as_deref()
        .map(|status| friend_status(Some(status), status != "offline"))
        .unwrap_or(PresenceStatus::Offline)
}

fn is_traveling(value: &Value) -> bool {
    json_string(
        value,
        &[
            "travelingToLocation",
            "presence.travelingToLocation",
            "presence.travelingToInstance",
        ],
    )
    .is_some_and(|location| !location.is_empty())
}

fn status_dot(status: PresenceStatus) -> Element<'static, Message> {
    let color = match status {
        PresenceStatus::Online => Color::from_rgb8(52, 199, 89),
        PresenceStatus::JoinMe => Color::from_rgb8(64, 156, 255),
        PresenceStatus::AskMe => Color::from_rgb8(255, 159, 10),
        PresenceStatus::DoNotDisturb => Color::from_rgb8(255, 69, 58),
        PresenceStatus::Offline => Color::from_rgb8(142, 142, 147),
    };
    container(Space::new())
        .width(Length::Fixed(10.0))
        .height(Length::Fixed(10.0))
        .style(move |_| {
            container::Style::default()
                .background(color)
                .border(Border {
                    width: 0.0,
                    radius: 5.0.into(),
                    color,
                })
        })
        .into()
}

fn resource_trust_rank(value: &Value) -> Option<String> {
    if let Some(rank) = json_string(value, &["trustRank", "tags.trustRank"]) {
        return Some(
            match rank.as_str() {
                "newUser" => "new",
                "knownUser" => "known",
                "trustedUser" => "trusted",
                "user" => "user",
                "visitor" => "visitor",
                other => other,
            }
            .to_string(),
        );
    }

    let tags = value
        .get("tags")
        .and_then(|tags| tags.get("raw").or(Some(tags)))
        .and_then(Value::as_array)?;
    [
        ("system_trust_veteran", "trusted"),
        ("system_trust_trusted", "known"),
        ("system_trust_known", "user"),
        ("system_trust_basic", "new"),
    ]
    .into_iter()
    .find_map(|(tag, rank)| {
        tags.iter()
            .any(|value| value.as_str() == Some(tag))
            .then(|| rank.to_string())
    })
    .or_else(|| Some("visitor".to_string()))
}

fn resource_avatar_performance(value: &Value) -> Option<AvatarPerformance> {
    if !string_field(value, &["id", "identifier.id"]).starts_with("avtr_") {
        return None;
    }

    let mut ratings = Vec::new();
    if let Some(performance) = value.get("performance").and_then(Value::as_object) {
        ratings.extend(
            performance
                .iter()
                .filter(|(key, _)| !key.ends_with("_sort") && !key.ends_with("Sort"))
                .filter_map(|(_, rating)| rating.as_str())
                .filter_map(parse_avatar_performance),
        );
    }
    ratings.extend(
        value
            .get("unityPackages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|package| package.get("performanceRating").and_then(Value::as_str))
            .filter_map(parse_avatar_performance),
    );

    ratings.into_iter().max()
}

fn parse_avatar_performance(value: &str) -> Option<AvatarPerformance> {
    match value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
        .as_str()
    {
        "excellent" => Some(AvatarPerformance::Excellent),
        "good" => Some(AvatarPerformance::Good),
        "medium" => Some(AvatarPerformance::Medium),
        "poor" => Some(AvatarPerformance::Poor),
        "verypoor" => Some(AvatarPerformance::VeryPoor),
        _ => None,
    }
}

fn item_card_style(
    item: &ResultItem,
    hovered: bool,
) -> impl Fn(&Theme) -> container::Style + use<> {
    let performance = resource_avatar_performance(&item.raw);
    let rank = item.trust_rank.clone();
    move |_| {
        let color = if let Some(performance) = performance {
            match performance {
                AvatarPerformance::Excellent => Color::from_rgb8(8, 43, 27),
                AvatarPerformance::Good => Color::from_rgb8(14, 58, 30),
                AvatarPerformance::Medium => Color::from_rgb8(61, 38, 13),
                AvatarPerformance::Poor => Color::from_rgb8(60, 18, 20),
                AvatarPerformance::VeryPoor => Color::from_rgb8(36, 8, 12),
            }
        } else {
            rank_color(rank.as_deref())
        };
        bordered_card_style(color, hovered)
    }
}

fn rank_card_style(rank: Option<String>, hovered: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| bordered_card_style(rank_color(rank.as_deref()), hovered)
}

fn related_card_style(hovered: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| bordered_card_style(Color::from_rgb8(20, 22, 32), hovered)
}

fn rank_color(rank: Option<&str>) -> Color {
    match rank {
        Some("visitor") => Color::from_rgb8(24, 26, 31),
        Some("new") => Color::from_rgb8(12, 27, 45),
        Some("user") => Color::from_rgb8(11, 36, 27),
        Some("known") => Color::from_rgb8(38, 26, 12),
        Some("trusted") => Color::from_rgb8(31, 18, 44),
        _ => Color::from_rgb8(19, 21, 31),
    }
}

fn bordered_card_style(color: Color, hovered: bool) -> container::Style {
    let background = if hovered {
        lighten_color(color, 0.14)
    } else {
        color
    };
    container::Style::default()
        .background(background)
        .border(Border {
            width: 1.0,
            radius: 6.0.into(),
            color: if hovered {
                Color::from_rgb8(72, 174, 193)
            } else {
                Color::from_rgb8(47, 50, 70)
            },
        })
}

fn search_modal_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(12, 13, 18))
        .border(Border {
            width: 1.0,
            radius: 8.0.into(),
            color: Color::from_rgb8(55, 58, 76),
        })
}

fn account_menu_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Color {
            r: 0.11,
            g: 0.11,
            b: 0.12,
            a: 0.95,
        })
        .border(Border {
            width: 1.0,
            radius: 8.0.into(),
            color: Color::from_rgb8(72, 72, 78),
        })
}

fn search_dim_style(_: &Theme) -> container::Style {
    container::Style::default().background(Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.62,
    })
}

fn lighten_color(color: Color, amount: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * amount,
        g: color.g + (1.0 - color.g) * amount,
        b: color.b + (1.0 - color.b) * amount,
        a: color.a,
    }
}

fn resource_badges(value: &Value) -> Vec<String> {
    let mut badges = Vec::new();
    if json_bool(value, &["ageVerified"])
        || json_string(value, &["ageVerificationStatus"])
            .is_some_and(|status| matches!(status.as_str(), "verified" | "18+"))
    {
        badges.push("18+ VERIFIED".to_string());
    }
    if json_bool(value, &["isFriend", "social.isFriend"]) {
        badges.push("FRIEND".to_string());
    }
    if value
        .get("isVerified")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        badges.push("VERIFIED".to_string());
    }
    if let Some(status) = json_string(value, &["status", "identity.status.status"]) {
        badges.push(status.to_uppercase());
    } else if let Some(status) = value.get("releaseStatus").and_then(Value::as_str) {
        badges.push(status.to_uppercase());
    } else if let Some(status) = value.get("membershipStatus").and_then(Value::as_str) {
        badges.push(status.to_uppercase());
    }
    badges
}

fn json_bool(value: &Value, paths: &[&str]) -> bool {
    paths.iter().any(|path| {
        path.split('.')
            .try_fold(value, |value, key| value.get(key))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn json_string(value: &Value, paths: &[&str]) -> Option<String> {
    paths.iter().find_map(|path| {
        path.split('.')
            .try_fold(value, |value, key| value.get(key))
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn detail_fields(page: Page, value: &Value) -> Vec<(&'static str, String)> {
    let paths: &[(&str, &[&str])] = match page {
        Page::Users | Page::Friends => &[
            ("Display name", &["displayName", "identity.displayName"]),
            ("Username", &["username", "identity.username"]),
            ("Status", &["status", "identity.status.status"]),
            (
                "Status message",
                &["statusDescription", "identity.status.statusDescription"],
            ),
            ("Pronouns", &["pronouns", "profile.pronouns"]),
            (
                "Platform",
                &[
                    "last_platform",
                    "presence.lastPlatform",
                    "presence.platform",
                ],
            ),
            ("Location", &["location", "presence.location.location"]),
            ("Joined", &["date_joined", "metadata.dateJoined"]),
            ("Bio", &["bio", "profile.bio"]),
        ],
        Page::Avatars | Page::MyAvatars => &[
            ("Name", &["name", "identity.name"]),
            ("Author", &["authorName", "identity.author.displayName"]),
            ("Description", &["description", "content.description"]),
            ("Release", &["releaseStatus", "publication.releaseStatus"]),
            ("Version", &["version", "identity.version"]),
            ("Created", &["created_at", "publication.createdAt"]),
            ("Updated", &["updated_at", "publication.updatedAt"]),
        ],
        Page::Groups => &[
            ("Name", &["name"]),
            ("Short code", &["shortCode"]),
            ("Description", &["description"]),
            ("Members", &["memberCount"]),
            ("Online", &["onlineMemberCount"]),
            ("Membership", &["membershipStatus"]),
            ("Privacy", &["privacy"]),
            ("Created", &["createdAt"]),
        ],
        Page::Worlds => &[
            ("Name", &["name", "identifier.name"]),
            ("Author", &["authorName", "identifier.author.displayName"]),
            ("Description", &["description"]),
            ("Capacity", &["capacity", "capacity.capacity"]),
            (
                "Recommended",
                &["recommendedCapacity", "capacity.recommendedCapacity"],
            ),
            ("Favorites", &["favorites", "stats.favorites"]),
            ("Occupants", &["occupants", "stats.occupants"]),
            ("Release", &["releaseStatus", "publications.releaseStatus"]),
            (
                "Published",
                &["publicationDate", "publications.publicationDate"],
            ),
        ],
        _ => &[
            ("Name", &["name", "displayName", "title"]),
            ("Description", &["description", "statusDescription"]),
            ("Type", &["type", "itemType", "storeType"]),
            ("Created", &["created_at", "createdAt", "created"]),
            ("Updated", &["updated_at", "updatedAt", "updated"]),
        ],
    };

    paths
        .iter()
        .filter_map(|(label, paths)| json_field(value, paths).map(|field| (*label, field)))
        .collect()
}

fn json_field(value: &Value, paths: &[&str]) -> Option<String> {
    let value = paths
        .iter()
        .find_map(|path| path.split('.').try_fold(value, |value, key| value.get(key)))?;
    match value {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(if *value { "Yes" } else { "No" }.to_string()),
        Value::Array(values) if !values.is_empty() => Some(
            values
                .iter()
                .filter_map(Value::as_str)
                .take(8)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        _ => None,
    }
}

async fn download_thumbnail(url: String) -> Result<Vec<u8>, String> {
    let cache_path = image_cache_path(&url)?;
    if let Ok(bytes) = tokio::fs::read(&cache_path).await {
        if !bytes.is_empty() {
            tracing::trace!(url, path = %cache_path.display(), "image loaded from disk cache");
            return Ok(bytes);
        }
    }

    let response = image_http_client()
        .get(&url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let source = response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| error.to_string())?;
    let bytes = tokio::task::spawn_blocking(move || resize_thumbnail(&source))
        .await
        .map_err(|error| error.to_string())??;
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    if let Err(error) = tokio::fs::write(&cache_path, &bytes).await {
        tracing::warn!(url, path = %cache_path.display(), %error, "image cache write failed");
    }
    Ok(bytes)
}

fn image_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("vrcx-rs/0.1")
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("image HTTP client configuration is valid")
    })
}

fn resize_thumbnail(source: &[u8]) -> Result<Vec<u8>, String> {
    let image = ::image::load_from_memory(source).map_err(|error| error.to_string())?;
    let thumbnail = image.thumbnail(THUMBNAIL_MAX_EDGE, THUMBNAIL_MAX_EDGE);
    let mut output = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut output, ::image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(output.into_inner())
}

fn image_cache_path(url: &str) -> Result<PathBuf, String> {
    let directories = directories::ProjectDirs::from("dev", "vrcx-rs", "VRCX Rust")
        .ok_or_else(|| "image cache directory unavailable".to_string())?;
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    Ok(directories
        .cache_dir()
        .join("images-v3")
        .join(format!("{:016x}.img", hasher.finish())))
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn two_factor_label(method: &TwoFactorMethod) -> &'static str {
    match method {
        TwoFactorMethod::Totp => "Authenticator",
        TwoFactorMethod::EmailOtp => "Email",
        TwoFactorMethod::RecoveryCode => "Recovery",
        TwoFactorMethod::Unknown(_) => "Other",
    }
}

fn resource_action(page: Page) -> Option<&'static str> {
    match page {
        Page::Users => Some("Add friend"),
        Page::Groups => Some("Join"),
        Page::Avatars | Page::MyAvatars => Some("Select"),
        Page::Inventory => Some("Consume"),
        Page::Files | Page::Props => Some("Delete"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AvatarPerformance, PresenceStatus, compact_json, favorite_display_groups,
        friend_secondary_text, friend_status, is_traveling, pipeline_event_summary,
        public_instance_location, resource_avatar_performance, resource_badges,
        resource_trust_rank, user_status_from_value,
    };
    use crate::models::favorite::{Favorite, FavoriteGroup, FavoriteType};
    use crate::store::FriendPresence;
    use crate::websocket::{PipelineEvent, event::FriendOnlineContent};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn favorite_tags_create_distinct_categories_even_without_matching_metadata() {
        let groups = vec![FavoriteGroup {
            id: "fvgrp_1".to_string(),
            name: "custom-internal-name".to_string(),
            display_name: "Proches".to_string(),
            owner_id: "usr_1".to_string(),
            owner_display_name: String::new(),
            favorite_type: FavoriteType::Friend,
            visibility: "private".to_string(),
            tags: vec!["group_0".to_string()],
        }];
        let first = Favorite {
            id: "fvrt_1".to_string(),
            favorite_id: "usr_2".to_string(),
            favorite_type: FavoriteType::Friend,
            tags: vec!["group_0".to_string()],
        };
        let second = Favorite {
            id: "fvrt_2".to_string(),
            favorite_id: "usr_3".to_string(),
            favorite_type: FavoriteType::Friend,
            tags: vec!["group_1".to_string()],
        };

        assert_eq!(favorite_display_groups(&first, &groups), vec!["Proches"]);
        assert_eq!(favorite_display_groups(&second, &groups), vec!["group_1"]);
    }

    #[test]
    fn separates_public_instances_and_prefers_custom_status_text() {
        let public = FriendPresence {
            online: true,
            location: Some("wrld_1:123~region(eu)".to_string()),
            status_description: Some("Je construis un avatar".to_string()),
            ..FriendPresence::default()
        };
        assert_eq!(
            public_instance_location(&public),
            Some("wrld_1:123~region(eu)")
        );
        assert_eq!(
            friend_secondary_text(&public, &HashMap::new()).as_deref(),
            Some("Je construis un avatar")
        );

        let private = FriendPresence {
            online: true,
            location: Some("wrld_1:123~private(usr_1)".to_string()),
            ..FriendPresence::default()
        };
        assert!(public_instance_location(&private).is_none());
    }

    #[test]
    fn activity_summary_does_not_expose_profile_payload() {
        let event = PipelineEvent::FriendOnline(FriendOnlineContent {
            user_id: "usr_123".to_string(),
            platform: "android".to_string(),
            location: "private".to_string(),
            can_request_invite: true,
            user: json!({
                "displayName": "Alice",
                "bio": "private profile content",
                "tags": ["private-tag"]
            }),
        });

        let (title, detail) = pipeline_event_summary(&event, &std::collections::HashMap::new());

        assert_eq!(title, "Friend online");
        assert_eq!(detail, "Alice on android");
        assert!(!detail.contains("private profile content"));
        assert!(!detail.contains("private-tag"));
    }

    #[test]
    fn compact_json_truncates_unicode_on_character_boundaries() {
        let value = json!({"bio": "é".repeat(300)});
        let compact = compact_json(&value);

        assert!(compact.ends_with("..."));
        assert!(compact.chars().count() <= 240);
    }

    #[test]
    fn user_badges_include_age_verification_and_friendship() {
        let value = json!({
            "ageVerified": true,
            "social": {"isFriend": true},
            "identity": {"status": {"status": "active"}}
        });

        assert_eq!(
            resource_badges(&value),
            vec!["18+ VERIFIED", "FRIEND", "ACTIVE"]
        );
    }

    #[test]
    fn maps_vrchat_presence_statuses() {
        assert_eq!(friend_status(Some("active"), true), PresenceStatus::Online);
        assert_eq!(friend_status(Some("joinMe"), true), PresenceStatus::JoinMe);
        assert_eq!(friend_status(Some("askMe"), true), PresenceStatus::AskMe);
        assert_eq!(
            friend_status(Some("busy"), true),
            PresenceStatus::DoNotDisturb
        );
        assert_eq!(
            user_status_from_value(&json!({"status": "offline"})),
            PresenceStatus::Offline
        );
    }

    #[test]
    fn maps_trust_rank_and_travel_state() {
        let user = json!({
            "tags": {"trustRank": "trustedUser"},
            "travelingToLocation": "wrld_123:instance"
        });

        assert_eq!(resource_trust_rank(&user).as_deref(), Some("trusted"));
        assert!(is_traveling(&user));
    }

    #[test]
    fn avatar_performance_uses_worst_available_platform_rating() {
        let avatar = json!({
            "id": "avtr_123",
            "performance": {
                "android": "VeryPoor",
                "ios": "good",
                "standalonewindows": "Poor",
                "android_sort": 1
            }
        });

        assert_eq!(
            resource_avatar_performance(&avatar),
            Some(AvatarPerformance::VeryPoor)
        );
    }
}
