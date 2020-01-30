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

    pub async fn get_stations_by_identifiers(self, identifiers: Vec<StationIdentifier>) -> Result<Vec<Station>, Error> {
        let mut stations = Vec::new();

        for identifier in identifiers {
            let url = self.build_url(&format!("{}{}", STATION_BY_UUID, identifier.stationuuid), None)?;
            debug!("Request station by UUID URL: {}", url);
            let data = self.send_message(url).await?;

            // Parse text to Vec<Station>
            let mut s: Vec<Station> = serde_json::from_str(data.as_str())?;
            stations.append(&mut s);
        }

        debug!("Found {} station(s)!", stations.len());
        Ok(stations)
    }

    // Create and send soup message, return the received data.
    async fn send_message(&self, url: Url) -> std::result::Result<String, Error> {
        let mut res = surf::get(url).await.map_err(|_| Error::SurfError)?;
        Ok(res.body_string().await.map_err(|_| Error::SurfError)?)
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        let mut url = self.server.join(param)?;
        options.map(|options| url.set_query(Some(options)));
        Ok(url)
    }
}
