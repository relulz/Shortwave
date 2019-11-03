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
use crate::settings::{Key, SettingsManager};
use crate::ui::{Notification, StationFlowBox};
use crate::utils::{Order, Sorting};

pub struct Library {
    pub widget: gtk::Box,
    flowbox: Rc<StationFlowBox>,
    library_stack: gtk::Stack,

    discover_button: gtk::Button,
    import_button: gtk::Button,

    client: Client,
    sender: Sender<Action>,
}

impl Library {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/library.ui");
        get_widget!(builder, gtk::Box, library);
        get_widget!(builder, gtk::Box, content_box);
        get_widget!(builder, gtk::Stack, library_stack);
        get_widget!(builder, gtk::Button, discover_button);
        get_widget!(builder, gtk::Button, import_button);

        get_widget!(builder, gtk::Image, logo_image);
        logo_image.set_from_icon_name(Some(config::APP_ID), gtk::IconSize::__Unknown(256));
        get_widget!(builder, gtk::Label, welcome_text);
        welcome_text.set_text(format!("Welcome to {}", config::NAME).as_str());

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        content_box.add(&flowbox.widget);

        let client = Client::new(Url::parse(&SettingsManager::get_string(Key::ApiServer)).unwrap());

        let library = Self {
            widget: library,
            flowbox,
            library_stack,
            discover_button,
            import_button,
            client,
            sender,
        };

        library.setup_signals();
        library.load_stations();
        library
    }

    fn setup_signals(&self) {
        let sender = self.sender.clone();
        self.discover_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowDiscover).unwrap();
        });

        let sender = self.sender.clone();
        self.import_button.connect_clicked(move |_| {
            sender.send(Action::LibraryGradioImport).unwrap();
        });
    }

    pub fn add_stations(&self, stations: Vec<Station>) {
        debug!("Add {} station(s)", stations.len());
        self.flowbox.add_stations(stations.clone());
        for station in stations {
            let id = StationIdentifier::from_station(&station);
            queries::insert_station_identifier(id).unwrap();
        }
        Self::update_stack_page(&self.library_stack);
    }

    pub fn remove_stations(&self, stations: Vec<Station>) {
        debug!("Remove {} station(s)", stations.len());
        self.flowbox.remove_stations(stations.clone());
        for station in stations {
            let id = StationIdentifier::from_station(&station);
            queries::delete_station_identifier(id).unwrap();
        }
        Self::update_stack_page(&self.library_stack);
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

    fn update_stack_page(library_stack: &gtk::Stack) {
        let ids = queries::get_station_identifiers().unwrap();
        if ids.len() == 0 {
            library_stack.set_visible_child_name("empty");
        } else {
            library_stack.set_visible_child_name("content");
        }
    }

    fn load_stations(&self) {
        // Print database info
        info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
        info!("Stations: {}", queries::get_station_identifiers().unwrap().len());

        // Load database async
        let identifiers = queries::get_station_identifiers().unwrap();
        let ctx = glib::MainContext::default();

        let flowbox = self.flowbox.clone();
        let library_stack = self.library_stack.clone();
        let sender = self.sender.clone();
        let fut = self.client.clone().get_stations_by_identifiers(identifiers).map(move |stations| {
            Self::update_stack_page(&library_stack);
            match stations {
                Ok(stations) => {
                    flowbox.add_stations(stations);
                }
                Err(err) => {
                    let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                    sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
                }
            }
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
