use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;
use url::Url;

#[derive(Deserialize, Debug, Clone, Eq, Hash)]
pub struct Station {
    #[serde(deserialize_with = "str_to_i32")]
    pub id: i32,
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    #[serde(deserialize_with = "str_to_url")]
    pub url: Option<Url>,
    #[serde(deserialize_with = "str_to_url")]
    pub homepage: Option<Url>,
    #[serde(deserialize_with = "str_to_url")]
    pub favicon: Option<Url>,
    pub tags: String,
    pub country: String,
    pub state: String,
    pub language: String,
    #[serde(deserialize_with = "str_to_i32")]
    pub votes: i32,
    pub negativevotes: String,
    pub lastchangetime: String,
    pub ip: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    #[serde(deserialize_with = "str_to_bool")]
    pub lastcheckok: bool,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    #[serde(deserialize_with = "str_to_i32")]
    pub clickcount: i32,
    #[serde(deserialize_with = "str_to_i32")]
    pub clicktrend: i32,
}

impl PartialEq for Station {
    fn eq(&self, other: &Station) -> bool {
        self.id == other.id
    }
}

fn str_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    i32::from_str(&s).map_err(de::Error::custom)
}

fn str_to_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Url::from_str(&s).ok())
}

fn str_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "0" {
        Ok(false)
    } else {
        Ok(true)
    }
}
