#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Flags {
    Active,
    Permanent,
    HardClose,
    Strict,
    QueueEnabled,
    InstancePersistenceEnabled,
    PlayerPersistenceEnabled,
    RoleRestricted,
}
