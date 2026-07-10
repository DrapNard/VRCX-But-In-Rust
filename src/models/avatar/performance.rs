#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PerformanceRating {
    #[serde(alias = "Excellent")]
    Excellent,
    #[serde(alias = "Good")]
    Good,
    #[serde(alias = "Medium")]
    Medium,
    #[serde(alias = "Poor")]
    Poor,
    #[serde(alias = "VeryPoor", alias = "very poor")]
    VeryPoor,
    #[serde(alias = "Unknown")]
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

#[cfg(test)]
mod tests {
    use super::{Performance, PerformanceRating};

    #[test]
    fn decodes_pascal_case_very_poor_rating() {
        let performance: Performance = serde_json::from_str(
            r#"{"android":"VeryPoor","ios":"VeryPoor","standaloneWindows":"Poor"}"#,
        )
        .unwrap();

        assert!(matches!(
            performance.android,
            Some(PerformanceRating::VeryPoor)
        ));
        assert!(matches!(performance.ios, Some(PerformanceRating::VeryPoor)));
        assert!(matches!(
            performance.standalone_windows,
            Some(PerformanceRating::Poor)
        ));
    }
}
