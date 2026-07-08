use crate::models::users::User;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthToken {
    pub ok: bool,
    pub token: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUser {
    pub user: User,
    pub requires_two_factor_auth: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorStatus {
    pub enabled: bool,
    pub verified: bool,
}
