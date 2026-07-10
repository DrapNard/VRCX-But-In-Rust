use super::{Asset, Economy, Identity, Media, Performance, Publication, Styles, UnityPackage};
use crate::models::users::UserSummary;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar {
    pub acknowledgements: Option<String>,
    pub identity: Identity,
    pub author: UserSummary,
    pub asset: Asset,
    pub media: Media,
    pub publication: Publication,
    pub economy: Economy,
    pub performance: Performance,
    pub styles: Styles,
    pub tags: Vec<String>,
    pub unity_packages: Vec<UnityPackage>,
}
