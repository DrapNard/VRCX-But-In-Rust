use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capacity {
    pub capacity: u16,
    pub recommended_capacity: u16,
}

impl Capacity {
    pub fn new(capacity: u16, recommended_capacity: u16) -> Self {
        Self {
            capacity,
            recommended_capacity,
        }
    }
}

impl<'de> Deserialize<'de> for Capacity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Number(number) => Ok(Self {
                capacity: number.as_u64().unwrap_or_default() as u16,
                recommended_capacity: 0,
            }),
            Value::Object(map) => Ok(Self {
                capacity: map
                    .get("capacity")
                    .and_then(Value::as_u64)
                    .unwrap_or_default() as u16,
                recommended_capacity: map
                    .get("recommendedCapacity")
                    .and_then(Value::as_u64)
                    .unwrap_or_default() as u16,
            }),
            _ => Ok(Self::default()),
        }
    }
}
