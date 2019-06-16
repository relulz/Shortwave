use glib::Sender;
use soup::prelude::*;
use soup::MessageExt;
use soup::Session;
use url::Url;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::*;
use crate::config;
use crate::model::StationModel;

pub struct Client {
    session: Session,
    server: Url,

    pub model: Rc<RefCell<StationModel>>,
}

impl Client {
    pub fn new(server: Url) -> Self {
        let user_agent = format!("{}/{}", config::NAME, config::VERSION);

        let session = soup::Session::new();
        session.set_property_user_agent(Some(&user_agent));

        let model = Rc::new(RefCell::new(StationModel::new()));
        debug!("Initialized new soup session with user agent \"{}\"", user_agent);

        Client { server, session, model }
    }

    pub fn send_station_request(&self, request: &StationRequest) {
        let url = self.build_url(STATION_SEARCH, Some(&request.url_encode()));
        debug!("Station request URL: {}", url);

        // Create SOUP message
        let message = soup::Message::new("GET", &url.to_string()).unwrap();

        // Send created message
        let model = self.model.clone();
        self.session.queue_message(&message, move |_, response| {
            model.borrow_mut().clear();

            let response_data = response.get_property_response_body_data().unwrap();
            let response_text = std::str::from_utf8(&response_data).unwrap();

            // Parse result text
            let result: Vec<Station> = serde_json::from_str(response_text).unwrap();
            debug!("Found {} station(s)!", result.len());

            for station in result {
                model.borrow_mut().add_station(station);
            }
        });
    }

    pub fn get_stream_url(&self, station: &Station, sender: Sender<String>) {
        let url = self.build_url(&format!("{}{}", PLAYABLE_STATION_URL, station.id), None);
        debug!("Request playable URL: {}", url);

        let message = soup::Message::new("GET", &url.to_string()).unwrap();
        let sender = sender.clone();
        self.session.queue_message(&message, move |_, response| {
            let response_data = response.get_property_response_body_data().unwrap();
            let response_text = std::str::from_utf8(&response_data).unwrap();

            // Parse result text
            let result: Vec<StationUrl> = serde_json::from_str(response_text).unwrap();
            debug!("Playable URL is: {}", result[0].url);
            sender.send(result[0].url.clone()).unwrap();
        });
    }

    fn build_url(&self, param: &str, options: Option<&str>) -> Url {
        let mut url = self.server.join(param).unwrap();
        options.map(|options| url.set_query(Some(options)));
        url
    }
}
