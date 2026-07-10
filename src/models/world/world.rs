use crate::models::instance::InstanceSummary;
use crate::models::users::identity::{Identity as UserIdentity, StatusInfo, UserStatus};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use super::publication::StatusRelease;
use super::{Capacity, Content, Identity, Media, Publication, Stats, Tags};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummary {
    pub identifier: Identity,
    pub media: Media,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub identifier: Identity,
    pub content: Content,
    pub media: Media,
    pub publications: Publication,
    pub stats: Stats,
    pub capacity: Capacity,
    pub tags: Tags,
    pub instance: Option<InstanceSummary>,
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let string = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let number = |key: &str| value.get(key).and_then(Value::as_u64).unwrap_or_default();
        let raw_tags = value
            .get("tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            identifier: Identity {
                id: string("id"),
                name: string("name"),
                namespace: string("namespace"),
                version: string("version"),
                store_id: string("storeId"),
                author: UserIdentity {
                    id: string("authorId"),
                    username: string("authorName"),
                    display_name: string("authorName"),
                    status: StatusInfo {
                        status: UserStatus::Unknown,
                        status_description: String::new(),
                    },
                },
            },
            content: Content {
                default_content_settings: Vec::new(),
                unity_package: value.get("unityPackages").cloned().unwrap_or(Value::Null),
                udon_products: value.get("udonProducts").cloned().unwrap_or(Value::Null),
            },
            media: Media {
                image_url: string("imageUrl"),
                thumbnail_image_url: string("thumbnailImageUrl"),
                preview_youtube_id: string("previewYoutubeId"),
                url_lists: value
                    .get("urlList")
                    .or_else(|| value.get("urlLists"))
                    .and_then(Value::as_array)
                    .map(|urls| {
                        urls.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            publications: Publication {
                release_status: match value.get("releaseStatus").and_then(Value::as_str) {
                    Some("public") => StatusRelease::Public,
                    Some("private") => StatusRelease::Private,
                    _ => StatusRelease::Unknown,
                },
                featured: value
                    .get("featured")
                    .and_then(Value::as_bool)
                    .unwrap_or_default(),
                publication_date: optional_string(&value, "publicationDate"),
                labs_publication_date: optional_string(&value, "labsPublicationDate"),
                created_at: string("created_at"),
                updated_at: string("updated_at"),
            },
            stats: Stats {
                favorites: number("favorites") as u32,
                visits: number("visits") as u32,
                heat: number("heat") as u16,
                popularity: number("popularity") as u8,
                occupants: number("occupants") as u32,
                public_occupants: number("publicOccupants") as u32,
                private_occupants: number("privateOccupants") as u32,
            },
            capacity: Capacity::new(
                number("capacity") as u16,
                number("recommendedCapacity") as u16,
            ),
            tags: Tags {
                content_warnings: Vec::new(),
                system_flags: Vec::new(),
                author_tags: Vec::new(),
                jam_tags: Vec::new(),
                admin_tags: Vec::new(),
                feature_tags: Vec::new(),
                event_tags: Vec::new(),
                raw: raw_tags,
            },
            instance: value
                .get("instance")
                .filter(|instance| !instance.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(serde::de::Error::custom)?,
        })
    }
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::World;

    #[test]
    fn decodes_flat_world_response() {
        let raw = r#"{
            "authorId":"usr_author",
            "authorName":"spacebaar",
            "capacity":80,
            "created_at":"2023-12-08T11:02:11.556Z",
            "defaultContentSettings":{},
            "favorites":405801,
            "heat":8,
            "id":"wrld_266523e8-9161-40da-acd0-6bd82e075833",
            "imageUrl":"https://example.com/image.png",
            "labsPublicationDate":"2024-01-13T03:31:45.106Z",
            "name":"Popcorn Palace",
            "occupants":4343,
            "popularity":10,
            "previewYoutubeId":null,
            "publicationDate":"2024-01-13T03:31:45.107Z",
            "recommendedCapacity":80,
            "releaseStatus":"public",
            "tags":["author_tag_Chill","system_approved"],
            "thumbnailImageUrl":"https://example.com/thumb.png",
            "udonProducts":[],
            "unityPackages":[{"platform":"android"}],
            "updated_at":"2024-01-14T00:00:00.000Z"
        }"#;

        let world: World = serde_json::from_str(raw).unwrap();

        assert_eq!(
            world.identifier.id,
            "wrld_266523e8-9161-40da-acd0-6bd82e075833"
        );
        assert_eq!(world.identifier.author.display_name, "spacebaar");
        assert_eq!(world.capacity.capacity, 80);
        assert_eq!(world.capacity.recommended_capacity, 80);
        assert_eq!(world.media.preview_youtube_id, "");
        assert_eq!(world.tags.raw.len(), 2);
    }
}
