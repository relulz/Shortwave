use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::rc::Rc;

use crate::api::{Client, Station};
use crate::app::Action;
use crate::config;
use crate::database::connection;
use crate::database::gradio_db;
use crate::database::queries;
use crate::database::StationIdentifier;
use crate::settings::{settings_manager, Key};
use crate::ui::{Notification, StationFlowBox};
use crate::utils::{Order, Sorting};

pub struct Library {
    pub widget: gtk::Box,
    flowbox: Rc<StationFlowBox>,
    library_stack: gtk::Stack,

    client: Client,
    sender: Sender<Action>,
}

impl Library {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/library.ui");
        get_widget!(builder, gtk::Box, library);
        get_widget!(builder, gtk::Box, content_box);
        get_widget!(builder, gtk::Stack, library_stack);

        get_widget!(builder, gtk::Image, logo_image);
        logo_image.set_from_icon_name(Some(config::APP_ID), gtk::IconSize::__Unknown(256));
        get_widget!(builder, gtk::Label, welcome_text);
        welcome_text.set_text(format!("Welcome to {}", config::NAME).as_str());

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        content_box.add(&flowbox.widget);

        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());

        let library = Self {
            widget: library,
            flowbox,
            library_stack,
            client,
            sender,
        };

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
        let mut identifiers = queries::get_station_identifiers().unwrap();
        let ctx = glib::MainContext::default();

        let flowbox = self.flowbox.clone();
        let library_stack = self.library_stack.clone();
        let client = self.client.clone();
        let sender = self.sender.clone();

        let future = async move {
            // Shortwave Beta 1 or older saves the library data by using simple IDs.
            // Since January 2020 Shortwaves is using the new radio-browser.info API,
            // which is using UUIDs instead of IDs.
            // Instead of wiping the library data (that would be the easiest way), we're
            // migrating the data to UUIDs, so Shortwave Beta 1 users are keeping their
            // data. That also means, after Shortwave Beta 2 releases, we can remove that part
            // again.

            if gradio_db::is_id_db(&identifiers) {
                info!("Found old database type...");
                let old_identifiers = identifiers.clone();
                let mut new_identifiers = Vec::new();

                let spinner_notification = Notification::new_spinner("Migrating library data...");
                sender.send(Action::ViewShowNotification(spinner_notification.clone())).unwrap();
                info!("Start migration library data...");

                // Convert IDs to UUIDs
                for old_id in old_identifiers {
                    match gradio_db::id2uuid(old_id.clone()).await {
                        Ok(new_uuid) => match new_uuid {
                            Some(new_uuid) => new_identifiers.insert(0, new_uuid),
                            None => warn!("No UUID for ID \"{}\" found.", old_id.stationuuid),
                        },
                        Err(err) => {
                            let notification = Notification::new_error("Could not migrate library data to UUIDs.", &err.to_string());
                            sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
                            spinner_notification.hide();
                            return;
                        }
                    };
                }

                // Remove everything old
                for old_id in identifiers.clone() {
                    queries::delete_station_identifier(old_id).unwrap();
                }

                // Add new UUIDs identifiers
                for new_id in new_identifiers {
                    queries::insert_station_identifier(new_id.clone()).unwrap();
                    identifiers.insert(0, new_id);
                }

                info!("Migration done.");
                spinner_notification.hide();
            }
            // End of migration part //////////////////////////////////////

            let stations = client.clone().get_stations_by_identifiers(identifiers).await;
            Self::update_stack_page(&library_stack.clone());

            match stations {
                Ok(stations) => {
                    flowbox.add_stations(stations);
                }
                Err(err) => {
                    let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                    sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
                }
            }
        };

        ctx.spawn_local(future);
    }
}
