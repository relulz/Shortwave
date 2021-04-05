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

use async_std_resolver::resolver;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use once_cell::unsync::OnceCell;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use url::Url;

use std::net::IpAddr;
use std::rc::Rc;
use std::time::Duration;

use crate::api::*;
use crate::config;
use crate::database::StationIdentifier;
use crate::model::SwStationModel;

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

#[derive(Clone, Debug)]
pub struct Client {
    pub model: Rc<SwStationModel>,

    lookup_domain: String,
    server: OnceCell<Url>,
}

impl Client {
    pub fn new(lookup_domain: String) -> Self {
        Client {
            model: Rc::new(SwStationModel::new()),
            lookup_domain,
            server: OnceCell::default(),
        }
    }

    pub async fn send_station_request(self, request: StationRequest) -> Result<(), Error> {
        let url = self.build_url(STATION_SEARCH, Some(&request.url_encode())).await?;
        debug!("Station request URL: {}", url);
        let stations_md: Vec<StationMetadata> = HTTP_CLIENT.get_async(url.as_ref()).await?.json().await?;
        let stations: Vec<SwStation> = stations_md.into_iter().map(SwStation::new).collect();

        debug!("Found {} station(s)!", stations.len());
        self.model.clear();
        for station in &stations {
            self.model.add_station(station);
        }

        Ok(())
    }

    pub async fn get_station_by_identifier(self, identifier: StationIdentifier) -> Result<SwStation, Error> {
        let url = self.build_url(&format!("{}{}", STATION_BY_UUID, identifier.stationuuid), None).await?;
        debug!("Request station by UUID URL: {}", url);

        let stations_md: Vec<StationMetadata> = HTTP_CLIENT.get_async(url.as_ref()).await?.json().await?;
        let mut data: Vec<SwStation> = stations_md.into_iter().map(SwStation::new).collect();

        match data.pop() {
            Some(station) => Ok(station),
            None => {
                warn!("API: No station for identifier \"{}\" found", &identifier.stationuuid);
                Err(Error::InvalidStationError(identifier.stationuuid))
            }
        }
    }

    async fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        if self.server.get().is_none() {
            let server_ip = Self::get_api_server(self.lookup_domain.clone()).await.ok_or(Error::NoServerReachable)?;
            self.server.set(server_ip).unwrap();
        }

        let mut url = self.server.get().unwrap().join(param)?;
        if let Some(options) = options {
            url.set_query(Some(options))
        }
        Ok(url)
    }

    async fn get_api_server(lookup_domain: String) -> Option<Url> {
        let resolver = resolver(async_std_resolver::config::ResolverConfig::default(), async_std_resolver::config::ResolverOpts::default())
            .await
            .expect("failed to connect resolver");

        // Do forward lookup to receive a list with the api servers
        let response = resolver.lookup_ip(lookup_domain).await.ok()?;
        let mut ips: Vec<IpAddr> = response.iter().collect();

        // Shuffle it to make sure we're not using always the same one
        ips.shuffle(&mut thread_rng());

        for ip in ips {
            // Do a reverse lookup to get the hostname
            let result = resolver.reverse_lookup(ip).await.ok().and_then(|r| r.into_iter().next());
            if result.is_none() {
                warn!("Reverse lookup failed for {} failed", ip);
                continue;
            }
            let hostname = result.unwrap();

            // Check if the server is online / returns data
            // If not, try using the next one in the list
            debug!("Trying to connect to {} ({})", hostname.to_string(), ip.to_string());
            match Self::test_api_server(hostname.to_string()).await {
                Ok(_) => {
                    info!("Using {} ({}) as api sever", hostname.to_string(), ip.to_string());
                    return Some(Url::parse(&format!("https://{}/", hostname)).unwrap());
                }
                Err(err) => {
                    warn!("Unable to connect {}: {}", ip.to_string(), err.to_string());
                }
            }
        }

        None
    }

    async fn test_api_server(ip: String) -> Result<(), Error> {
        let _stats: Option<Stats> = HTTP_CLIENT.get_async(format!("https://{}/{}", ip, STATS)).await?.json().await?;
        Ok(())
    }
}
