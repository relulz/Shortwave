use glib::futures::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::io;
use std::rc::Rc;

use crate::api::{Client, Station};
use crate::app::Action;
use crate::config;
use crate::database::connection;
use crate::database::queries;
use crate::database::StationIdentifier;
use crate::ui::StationFlowBox;
use crate::utils::{Order, Sorting};

pub struct Library {
    pub widget: gtk::Box,
    flowbox: Rc<StationFlowBox>,

    client: Client,
    sender: Sender<Action>,
}

impl Library {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/library.ui");
        let widget: gtk::Box = builder.get_object("library").unwrap();
        let content_box: gtk::Box = builder.get_object("content_box").unwrap();

        let logo_image: gtk::Image = builder.get_object("logo_image").unwrap();
        logo_image.set_from_icon_name(Some(format!("{}-symbolic", config::APP_ID).as_str()), gtk::IconSize::__Unknown(128));
        let welcome_text: gtk::Label = builder.get_object("welcome_text").unwrap();
        welcome_text.set_text(format!("Welcome to {}", config::NAME).as_str());

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        flowbox.set_sorting(Sorting::Name, Order::Ascending);
        content_box.add(&flowbox.widget);

        let client = Client::new(Url::parse("http://www.radio-browser.info/webservice/").unwrap());

        let library = Self { widget, flowbox, client, sender };

        library.load_stations();
        library
    }

    pub fn add_stations(&self, stations: Vec<Station>) {
        debug!("Add {} station(s)", stations.len());
        self.flowbox.add_stations(stations.clone());
        for station in stations {
            let id = StationIdentifier::from_station(&station);
            queries::insert_station_identifier(id).unwrap();
        }
    }

    pub fn remove_stations(&self, stations: Vec<Station>) {
        debug!("Remove {} station(s)", stations.len());
        self.flowbox.remove_stations(stations.clone());
        for station in stations {
            let id = StationIdentifier::from_station(&station);
            queries::delete_station_identifier(id).unwrap();
        }
    }

    pub fn contains_station(station: &Station) -> bool {
        // Get station identifier
        let identifier = StationIdentifier::from_station(station);

        // Check if database contains this identifier
        let db = queries::get_station_identifiers().unwrap();
        db.contains(&identifier)
    }

    pub fn set_sorting(&self, sorting: Sorting, order: Order) {
        self.flowbox.set_sorting(sorting, order);
    }

    fn load_stations(&self) {
        // Print database info
        info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
        info!("Stations: {}", queries::get_station_identifiers().unwrap().len());

        // Load database async
        let identifiers = queries::get_station_identifiers().unwrap();
        let ctx = glib::MainContext::default();

        let flowbox = self.flowbox.clone();
        let fut = self.client.clone().get_stations_by_identifiers(identifiers).map(move |stations| {
            flowbox.add_stations(stations);
        });
        ctx.spawn_local(fut);
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum LibraryError {
        Io(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
        Restson(err: restson::Error) {
            from()
            description("restson error")
            display("Network error: {}", err)
            cause(err)
        }
        Serde(err: serde_json::error::Error) {
            from()
            description("serde error")
            display("Parser error: {}", err)
            cause(err)
        }
    }
}
