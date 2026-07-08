#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub balance: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceEarnings {
    pub available: u64,
    pub pending: u64,
    pub lifetime: u64,
}
