#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DefaultContentSettings {
    Drones,
    Emoji,
    Pedestals,
    Prints,
    Props,
    Stickers,
}
