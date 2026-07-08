#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionList {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub subscription_id: String,
    pub status: String,
    pub period: String,
    pub started_at: String,
    pub expires_at: Option<String>,
}
