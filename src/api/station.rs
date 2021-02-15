// Shortwave - station.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use serde::Deserialize;
use serde::Deserializer;
use std::str::FromStr;
use url::Url;

#[derive(Default, Debug, Clone, serde_derive::Deserialize)]
pub struct Station {
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    #[serde(deserialize_with = "str_to_url")]
    pub url: Option<Url>,
    #[serde(deserialize_with = "str_to_url")]
    pub url_resolved: Option<Url>,
    #[serde(deserialize_with = "str_to_url")]
    pub homepage: Option<Url>,
    #[serde(deserialize_with = "str_to_url")]
    pub favicon: Option<Url>,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub votes: i32,
    pub lastchangetime: String,
    pub codec: String,
    pub bitrate: i32,
    pub hls: i32,
    pub lastcheckok: i32,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub lastlocalchecktime: String,
    pub clicktimestamp: String,
    pub clickcount: i32,
    pub clicktrend: i32,
}

fn str_to_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Url::from_str(&s).ok())
}
