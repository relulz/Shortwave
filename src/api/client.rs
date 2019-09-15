use gio::prelude::*;
use gio::{NONE_CANCELLABLE, DataInputStream};
use glib::prelude::*;
use glib::GString;
use soup::prelude::*;
use soup::Session;
use url::Url;

use crate::api::*;
use crate::config;
use crate::database::StationIdentifier;

#[derive(Clone)]
pub struct Client {
    session: Session,
    server: Url,
}

impl Client {
    pub fn new(server: Url) -> Self {
        let user_agent = format!("{}/{}", config::NAME, config::VERSION);

        let session = soup::Session::new();
        session.set_property_user_agent(Some(&user_agent));
        debug!("Initialized new soup session with user agent \"{}\"", user_agent);

        Client { server, session }
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
            let url = self.build_url(&format!("{}{}", STATION_BY_ID, identifier.station_id), None)?;
            debug!("Request station by ID URL: {}", url);
            let data = self.send_message(url).await?;

            // Parse text to Vec<Station>
            let mut s: Vec<Station> = serde_json::from_str(data.as_str())?;
            stations.append(&mut s);
        }

        debug!("Found {} station(s)!", stations.len());
        Ok(stations)
    }

    pub async fn get_stream_url(self, station: Station) -> Result<StationUrl, Error> {
        let url = self.build_url(&format!("{}{}", PLAYABLE_STATION_URL, station.id), None)?;
        debug!("Request playable URL: {}", url);
        let data = self.send_message(url).await?;

        // Parse text to StationUrl
        let result: Vec<StationUrl> = serde_json::from_str(data.as_str())?;
        debug!("Playable URL is: {}", result[0].url);
        Ok(result[0].clone())
    }

    // Create and send soup message, return the received data.
    async fn send_message(&self, url: Url) -> std::result::Result<GString, Error> {
        // Create SOUP message
        dbg!(url.clone());
        match soup::Message::new("GET", &url.to_string()){
            Some(message) => {
                // Send created message
                let input_stream = self.session.send_async_future(&message).await?;

                // Create DataInputStream and read read received data
                let data_input_stream: DataInputStream = gio::DataInputStream::new(&input_stream);
                //TODO: Crash here, if stream is empty
                let result = data_input_stream.read_upto_async_future("", glib::PRIORITY_LOW).await?;

                Ok(result.0)
            },
            // Return error when message cannot be created
            None => Err(Error::SoupMessageError),
        }
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Result<Url, Error> {
        let mut url = self.server.join(param)?;
        options.map(|options| url.set_query(Some(options)));
        Ok(url)
    }
}
