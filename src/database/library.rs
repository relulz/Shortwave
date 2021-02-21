// Shortwave - library.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
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
use gtk::glib;
use gtk::prelude::*;

use futures::future::join_all;
use std::rc::Rc;

use crate::api::{Client, Error, SwStation};
use crate::app::Action;
use crate::database::connection;
use crate::database::queries;
use crate::database::StationIdentifier;
use crate::i18n::*;
use crate::model::SwStationModel;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;

pub struct Library {
    pub model: SwStationModel,

    client: Client,
    sender: Sender<Action>,
}

impl Library {
    pub fn new(sender: Sender<Action>) -> Self {
        let model = SwStationModel::new();
        let client = Client::new(settings_manager::get_string(Key::ApiLookupDomain));

        let library = Self { model, client, sender };

        library.load_stations();
        library
    }

    pub fn add_stations(&self, stations: Vec<SwStation>) {
        debug!("Add {} station(s)", stations.len());
        for station in stations {
            self.model.add_station(&station);

            let id = StationIdentifier::from_station(&station);
            queries::insert_station_identifier(id).unwrap();
        }
    }

    pub fn remove_stations(&self, stations: Vec<SwStation>) {
        debug!("Remove {} station(s)", stations.len());
        for station in stations {
            self.model.remove_station(&station);

            let id = StationIdentifier::from_station(&station);
            queries::delete_station_identifier(id).unwrap();
        }
    }

    pub fn contains_station(station: &SwStation) -> bool {
        // Get station identifier
        let identifier = StationIdentifier::from_station(station);

        // Check if database contains this identifier
        let db = queries::get_station_identifiers().unwrap();
        db.contains(&identifier)
    }

    fn load_stations(&self) {
        // Print database info
        info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
        info!("Stations: {}", queries::get_station_identifiers().unwrap().len());

        // Load database async
        let identifiers = queries::get_station_identifiers().unwrap();
        let model = self.model.clone();
        let client = self.client.clone();
        let sender = self.sender.clone();
        let future = async move {
            let mut futures = Vec::new();

            for id in identifiers {
                let future = client.clone().get_station_by_identifier(id);
                futures.insert(0, future);
            }
            let results = join_all(futures).await;

            for result in results {
                match result {
                    Ok(station) => model.add_station(&station),
                    Err(err) => match err {
                        Error::InvalidStationError(uuid) => {
                            let id = StationIdentifier::from_uuid(uuid);
                            queries::delete_station_identifier(id).unwrap();

                            let notification = Notification::new_info(&i18n("No longer existing station removed from library."));
                            send!(sender, Action::ViewShowNotification(notification));
                        }
                        _ => {
                            let notification = Notification::new_error(&i18n("Station data could not be received."), &err.to_string());
                            send!(sender, Action::ViewShowNotification(notification));
                        }
                    },
                }
            }
        };

        spawn!(future);
    }
}
