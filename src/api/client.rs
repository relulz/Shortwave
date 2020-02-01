use futures::future::join_all;
use futures_util::future::FutureExt;
use isahc::prelude::*;
use url::Url;

use std::cell::RefCell;
use std::rc::Rc;

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

    pub async fn get_stations_by_identifiers(self, identifiers: Vec<StationIdentifier>) -> Result<Vec<Station>, Error> {
        let stations = Rc::new(RefCell::new(Vec::new()));
        let mut futs = Vec::new();

        for identifier in identifiers {
            let url = self.build_url(&format!("{}{}", STATION_BY_UUID, identifier.stationuuid), None)?;
            debug!("Request station by UUID URL: {}", url);

            let stations = stations.clone();
            let fut = self.send_message(url).map(move |data| {
                // Parse text to Vec<Station>
                let mut s: Vec<Station> = serde_json::from_str(data.unwrap().as_str()).unwrap();
                stations.borrow_mut().append(&mut s);
            });

            // We're collecting the futures here, so we can execute them
            // later alltogether at the same time, instead of executing them separately.
            futs.insert(0, fut);
        }
        // Here we're are doing the real work. Executing all futures!
        join_all(futs).await;

        let stations: Vec<Station> = stations.borrow_mut().to_vec();
        debug!("Found {} station(s)!", stations.len());
        Ok(stations)
    }

    // Create and send message, return the received data.
    async fn send_message(&self, url: Url) -> Result<String, Error> {
        let response = isahc::get_async(url.to_string()).await?.text_async().await?;
        Ok(response)
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        let mut url = self.server.join(param)?;
        options.map(|options| url.set_query(Some(options)));
        Ok(url)
    }
}
