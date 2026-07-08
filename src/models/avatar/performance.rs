#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PerformanceRating {
    Excellent,
    Good,
    Medium,
    Poor,
    VeryPoor,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    pub android: Option<PerformanceRating>,
    pub android_sort: Option<u32>,
    pub ios: Option<PerformanceRating>,
    pub ios_sort: Option<u32>,
    pub standalone_windows: Option<PerformanceRating>,
    pub standalone_windows_sort: Option<u32>,
}
