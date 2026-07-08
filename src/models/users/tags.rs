#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrustRank {
    Visitor,
    NewUser,
    User,
    KnownUser,
    TrustedUser,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrollState {
    None,
    Probable,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupporterState {
    None,
    Supporter,
    EarlySupporter,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AdminTags {
    IsModerator,
    ShowModTag,
    HasAdminAvatarAccess,
    HasAdminWorldAccess,
    HasAdminCannyAccess,
    CanGrantLicenses,
    LockLevel,
    LockTags,
    HasOfficialThumbnail,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Permissions {
    HasWorldAccess,
    HasAvatarAccess,
    HasFeedbackAccess,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    pub trust_rank: TrustRank,
    pub troll_state: TrollState,
    pub admin_tags: Vec<AdminTags>,
    pub supporter_state: SupporterState,
    pub permissions: Vec<Permissions>,
    pub languages: Vec<String>,
    pub raw: Vec<String>,
}
