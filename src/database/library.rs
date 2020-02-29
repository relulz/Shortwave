// Shortwave - library.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::Sender;
use gtk::prelude::*;
use url::Url;

use futures::future::join_all;
use std::rc::Rc;

use crate::api::{Client, Station};
use crate::app::Action;
use crate::config;
use crate::database::connection;
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

        // keyboard focus
        get_widget!(builder, gtk::ScrolledWindow, scrolledwindow);
        flowbox.widget.set_focus_vadjustment(&scrolledwindow.get_vadjustment().unwrap());

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
        if ids.is_empty() {
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
        let flowbox = self.flowbox.clone();
        let library_stack = self.library_stack.clone();
        let client = self.client.clone();
        let sender = self.sender.clone();
        let future = async move {
            let mut stations = Vec::new();
            let mut futures = Vec::new();

            for id in identifiers {
                let future = client.clone().get_station_by_identifier(id);
                futures.insert(0, future);
            }
            let results = join_all(futures).await;

            for result in results {
                match result {
                    Ok(station) => stations.insert(0, station),
                    Err(err) => {
                        let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                        send!(sender, Action::ViewShowNotification(notification));
                        break;
                    }
                }
            }

            Self::update_stack_page(&library_stack.clone());
            flowbox.add_stations(stations);
        };

        spawn!(future);
    }
}
