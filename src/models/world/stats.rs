#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub favorites: u32,
    pub visits: u32,
    pub heat: u16,
    pub popularity: u8,
    pub occupants: u32,
    pub public_occupants: u32,
    pub private_occupants: u32,
}
