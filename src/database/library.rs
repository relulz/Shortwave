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
use crate::model::{Order, Sorting};
use crate::ui::StationFlowBox;

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
        content_box.add(&flowbox.widget);

        let client = Client::new(Url::parse("http://www.radio-browser.info/webservice/").unwrap());

        let library = Self { widget, flowbox, client, sender };

        library.check_database();
        library.update_flowbox();
        library
    }

    pub fn add_stations(&self, stations: Vec<Station>) {
        debug!("Add {} station(s)", stations.len());
        for station in stations {
            let id = StationIdentifier::new(&station);
            queries::insert_station_identifier(id).unwrap();
        }
        self.update_flowbox();
    }

    pub fn remove_stations(&self, stations: Vec<Station>) {
        debug!("Remove {} station(s)", stations.len());
        for station in stations {
            let id = StationIdentifier::new(&station);
            queries::delete_station_identifier(id).unwrap();
        }
        self.update_flowbox();
    }

    pub fn contains_station(station: &Station) -> bool {
        // Get station identifier
        let identifier = StationIdentifier::new(station);

        // Check if database contains this identifier
        let db = queries::get_station_identifiers().unwrap();
        db.contains(&identifier)
    }

    pub fn set_sorting(&self, sorting: Sorting, order: Order) {
        //self.library_model.borrow_mut().set_sorting(sorting, order);
    }

    fn update_flowbox(&self) {
        let identifiers = queries::get_station_identifiers().unwrap();

        let flowbox = self.flowbox.clone();
        let fut = self.client.clone().get_stations_by_identifiers(identifiers).map(move |stations| {
            flowbox.set_stations(stations);
        });

        let ctx = glib::MainContext::default();
        ctx.spawn_local(fut);
    }

    fn check_database(&self) {
        // Print database info
        info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
        info!("Stations: {}", queries::get_station_identifiers().unwrap().len());
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
