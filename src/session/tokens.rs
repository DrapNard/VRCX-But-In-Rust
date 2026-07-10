#[derive(Debug, Clone)]
pub struct SessionTokens {
    pub auth: Option<String>,
    pub two_factor_auth: Option<String>,
}
