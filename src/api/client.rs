use gio::prelude::*;
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

    pub async fn send_station_request(self, request: StationRequest) -> Vec<Station> {
        let url = self.build_url(STATION_SEARCH, Some(&request.url_encode()));
        debug!("Station request URL: {}", url);
        let data = self.send_message(url).await.unwrap().0;

        // Parse text to Vec<Station>
        let stations: Vec<Station> = serde_json::from_str(data.as_str()).unwrap();
        debug!("Found {} station(s)!", stations.len());

        stations
    }

    pub async fn get_stations_by_identifiers(self, identifiers: Vec<StationIdentifier>) -> Vec<Station> {
        let mut stations = Vec::new();

        for identifier in identifiers {
            let url = self.build_url(&format!("{}{}", STATION_BY_ID, identifier.station_id), None);
            debug!("Request station by ID URL: {}", url);
            let data = self.send_message(url).await.unwrap().0;

            // Parse text to Vec<Station>
            let mut s: Vec<Station> = serde_json::from_str(data.as_str()).unwrap();
            stations.append(&mut s);
        }

        debug!("Found {} station(s)!", stations.len());
        stations
    }

    pub async fn get_stream_url(self, station: Station) -> StationUrl {
        let url = self.build_url(&format!("{}{}", PLAYABLE_STATION_URL, station.id), None);
        debug!("Request playable URL: {}", url);
        let data = self.send_message(url).await.unwrap().0;

        // Parse text to StationUrl
        let result: Vec<StationUrl> = serde_json::from_str(data.as_str()).unwrap();
        debug!("Playable URL is: {}", result[0].url);
        result[0].clone()
    }

    // Create and send soup message, return the received data.
    async fn send_message(&self, url: Url) -> Result<(GString, usize), gio::Error> {
        // Create SOUP message
        let message = soup::Message::new("GET", &url.to_string()).unwrap();

        // Send created message
        let input_stream = self.session.send_async_future(&message).await.unwrap();

        // Create DataInputStream and read read received data
        let data_input_stream = gio::DataInputStream::new(&input_stream);
        data_input_stream.read_upto_async_future("", glib::PRIORITY_LOW).await
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Url {
        let mut url = self.server.join(param).unwrap();
        options.map(|options| url.set_query(Some(options)));
        url
    }
}
