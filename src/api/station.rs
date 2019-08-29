use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash)]
pub struct Station {
    #[serde(deserialize_with = "de_from_str")]
    pub id: i32,
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub state: String,
    pub language: String,
    #[serde(deserialize_with = "de_from_str")]
    pub votes: i32,
    pub negativevotes: String,
    pub lastchangetime: String,
    pub ip: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    pub lastcheckok: String,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    #[serde(deserialize_with = "de_from_str")]
    pub clickcount: i32,
    #[serde(deserialize_with = "de_from_str")]
    pub clicktrend: i32,
}

impl PartialEq for Station {
    fn eq(&self, other: &Station) -> bool {
        self.id == other.id
    }
}

fn de_from_str<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    i32::from_str(&s).map_err(de::Error::custom)
}
