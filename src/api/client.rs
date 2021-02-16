// Shortwave - client.rs
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

use isahc::prelude::*;
use url::Url;

use crate::api::*;
use crate::config;
use crate::database::StationIdentifier;
use isahc::config::RedirectPolicy;
use std::time::Duration;

lazy_static! {
    pub static ref USER_AGENT: String = format!("{}/{}-{}", config::PKGNAME, config::VERSION, config::PROFILE);
    pub static ref HTTP_CLIENT: isahc::HttpClient = isahc::HttpClientBuilder::new()
        // Limit to reduce ram usage. We don't need 250 concurrent connections
        .max_connections(8)
        // Icons are fetched from different urls.
        // There's a lot of probability we aren't going to reuse the same connection
        .connection_cache_size(8)
        .timeout(Duration::from_secs(15))
        .redirect_policy(RedirectPolicy::Follow)
        .default_header("User-Agent", USER_AGENT.as_str())
        .build()
        .unwrap();
}

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
        let stations: Vec<Station> = HTTP_CLIENT.get_async(url.as_ref()).await?.json().await?;
        debug!("Found {} station(s)!", stations.len());

        Ok(stations)
    }

    pub async fn get_station_by_identifier(self, identifier: StationIdentifier) -> Result<Station, Error> {
        let url = self.build_url(&format!("{}{}", STATION_BY_UUID, identifier.stationuuid), None)?;
        debug!("Request station by UUID URL: {}", url);

        let mut data: Vec<Station> = HTTP_CLIENT.get_async(url.as_ref()).await?.json().await?;

        match data.pop() {
            Some(station) => Ok(station),
            None => {
                warn!("API: No station for identifier \"{}\" found", &identifier.stationuuid);
                Err(Error::InvalidStationError(identifier.stationuuid))
            }
        }
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        let mut url = self.server.join(param)?;
        if let Some(options) = options {
            url.set_query(Some(options))
        }
        Ok(url)
    }
}
