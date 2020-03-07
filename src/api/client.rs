// Shortwave - client.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use isahc::prelude::*;
use url::Url;

use crate::api::*;
use crate::database::StationIdentifier;

#[derive(Clone)]
pub struct Client {
    server: Url,
}

impl Client {
    pub fn new(server: Url) -> Self {
        Client { server }
    }

    pub async fn send_station_request(self, request: StationRequest) -> Result<Vec<Station>, Error> {
        let url = self.build_url(STATION_SEARCH, Some(&request.url_encode()))?;
        debug!("Station request URL: {}", url);
        let data = self.send_message(url).await?;

        // Parse text to Vec<Station>
        let stations: Vec<Station> = serde_json::from_str(data.as_str())?;
        debug!("Found {} station(s)!", stations.len());

        Ok(stations)
    }

    pub async fn get_station_by_identifier(self, identifier: StationIdentifier) -> Result<Station, Error> {
        let url = self.build_url(&format!("{}{}", STATION_BY_UUID, identifier.stationuuid), None)?;
        debug!("Request station by UUID URL: {}", url);

        let data = self.send_message(url).await?;

        // Parse text to Vec<Station>
        let mut s: Vec<Station> = serde_json::from_str(data.as_str())?;
        match s.pop() {
            Some(station) => Ok(station),
            None => {
                warn!("API: No station for identifier \"{}\" found", &identifier.stationuuid);
                Err(Error::InvalidStationError(identifier.stationuuid))
            }
        }
    }

    // Create and send message, return the received data.
    async fn send_message(&self, url: Url) -> Result<String, Error> {
        let response = isahc::get_async(url.to_string()).await?.text_async().await?;
        Ok(response)
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        let mut url = self.server.join(param)?;
        if let Some(options) = options {
            url.set_query(Some(options))
        }
        Ok(url)
    }
}
