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

use super::models::StationEntry;
use crate::api::{Client, Error, StationMetadata, SwStation};
use crate::app::Action;
use crate::database::connection;
use crate::database::queries;
use crate::i18n::*;
use crate::model::SwStationModel;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;
use futures::future::join_all;
use glib::{clone, GEnum, ObjectExt, ParamSpec, Sender, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;

#[derive(Display, Copy, Debug, Clone, EnumString, PartialEq, GEnum)]
#[repr(u32)]
#[genum(type_name = "SwLibraryStatus")]
pub enum SwLibraryStatus {
    Loading,
    Content,
    Empty,
    Offline,
}

impl Default for SwLibraryStatus {
    fn default() -> Self {
        SwLibraryStatus::Loading
    }
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct SwLibrary {
        pub model: SwStationModel,
        pub status: RefCell<SwLibraryStatus>,

        pub client: Client,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwLibrary {
        const NAME: &'static str = "SwLibrary";
        type ParentType = glib::Object;
        type Type = super::SwLibrary;

        fn new() -> Self {
            let model = SwStationModel::new();
            let status = RefCell::default();
            let client = Client::new(settings_manager::string(Key::ApiLookupDomain));
            let sender = OnceCell::default();

            Self { model, status, client, sender }
        }
    }

    impl ObjectImpl for SwLibrary {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpec::new_object("model", "Model", "Model", SwStationModel::static_type(), glib::ParamFlags::READABLE),
                    ParamSpec::new_enum(
                        "status",
                        "Status",
                        "Status",
                        SwLibraryStatus::static_type(),
                        SwLibraryStatus::default() as i32,
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "model" => self.model.to_value(),
                "status" => self.status.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SwLibrary(ObjectSubclass<imp::SwLibrary>);
}

impl SwLibrary {
    pub fn new(sender: Sender<Action>) -> Self {
        let library = glib::Object::new::<Self>(&[]).unwrap();

        let imp = imp::SwLibrary::from_instance(&library);
        imp.sender.set(sender).unwrap();

        library.load_stations();
        library
    }

    pub fn model(&self) -> SwStationModel {
        self.property("model").unwrap().get().unwrap().unwrap()
    }

    pub fn status(&self) -> SwLibraryStatus {
        self.property("status").unwrap().get().unwrap().unwrap()
    }

    pub fn add_stations(&self, stations: Vec<SwStation>) {
        let imp = imp::SwLibrary::from_instance(self);

        debug!("Add {} station(s)", stations.len());
        for station in stations {
            imp.model.add_station(&station);

            let entry = StationEntry::for_station(&station);
            queries::insert_station(entry).unwrap();
        }

        self.update_library_status();
    }

    pub fn remove_stations(&self, stations: Vec<SwStation>) {
        let imp = imp::SwLibrary::from_instance(self);

        debug!("Remove {} station(s)", stations.len());
        for station in stations {
            imp.model.remove_station(&station);
            queries::delete_station(&station.uuid()).unwrap();
        }

        self.update_library_status();
    }

    pub fn contains_station(station: &SwStation) -> bool {
        queries::contains_station(&station.uuid()).unwrap()
    }

    fn update_library_status(&self) {
        let imp = imp::SwLibrary::from_instance(self);

        if imp.model.n_items() == 0 {
            *imp.status.borrow_mut() = SwLibraryStatus::Empty;
        } else {
            *imp.status.borrow_mut() = SwLibraryStatus::Content;
        }

        self.notify("status");
    }

    fn load_stations(&self) {
        // Load database async
        let future = clone!(@strong self as this => async move {
            let entries = queries::stations().unwrap();

            // Print database info
            info!("Database Path: {}", connection::DB_PATH.to_str().unwrap());
            info!("Stations: {}", entries.len());

            // Set library status to loading
            let imp = imp::SwLibrary::from_instance(&this);
            *imp.status.borrow_mut() = SwLibraryStatus::Loading;
            this.notify("status");

            let futures = entries.into_iter().map(|entry| this.load_station(entry));
            join_all(futures).await;

            this.update_library_status();
        });
        spawn!(future);
    }

    /// Try to add a station to the database.
    async fn load_station(&self, entry: StationEntry) {
        let imp = imp::SwLibrary::from_instance(&self);

        if entry.is_local {
            if let Some(data) = &entry.data {
                match self.load_station_metadata(&entry.uuid, data) {
                    Ok(metadata) => imp.model.add_station(&SwStation::new_local(&entry.uuid, metadata)),
                    Err(_) => self.delete_unknown_station(&entry.uuid),
                }
            } else {
                self.delete_unknown_station(&entry.uuid);
            }
        } else {
            match imp.client.clone().station_by_uuid(&entry.uuid).await {
                Ok(station) => {
                    // Cache data for future use
                    let entry = StationEntry::for_station(&station);
                    queries::update_station(entry).unwrap();

                    // Add station to the library
                    imp.model.add_station(&station)
                }
                Err(err) => {
                    warn!("Failed to fetch station: {}", entry.uuid);
                    warn!("Error while receiving: {}", err);
                    warn!("Trying to use cached data");

                    let removed_online = matches!(err, Error::InvalidStationError(_));

                    if let Some(data) = &entry.data {
                        match self.load_station_metadata(&entry.uuid, data) {
                            Ok(metadata) => imp.model.add_station(&SwStation::new(metadata)),
                            Err(_) => {
                                if removed_online {
                                    self.delete_unknown_station(&entry.uuid);
                                } else {
                                    warn!("Ignoring station to try again next time");
                                }
                            }
                        }
                    } else if removed_online {
                        self.delete_unknown_station(&entry.uuid);
                    } else {
                        warn!("Ignoring station to try again next time");
                    }
                }
            }
        }
    }

    /// Deserialize the provided data as a station.
    fn load_station_metadata(&self, uuid: &str, data: &str) -> Result<StationMetadata, serde_json::Error> {
        match serde_json::from_str(data) {
            Ok(metadata) => Ok(metadata),
            Err(err) => {
                warn!("Failed to deserialize station: {}", uuid);
                warn!("Data from database: '{}'", data);
                warn!("Error while deserializing: {}", err);
                Err(err)
            }
        }
    }

    /// Delete an unknown or malformed station entry notifying the user.
    fn delete_unknown_station(&self, uuid: &str) {
        let imp = imp::SwLibrary::from_instance(&self);

        warn!("Removing unknown station: {}", uuid);
        queries::delete_station(&uuid).unwrap();

        let notification = Notification::new_info(&i18n("An invalid station was removed from the library."));
        send!(imp.sender.get().unwrap(), Action::ViewShowNotification(notification));
    }
}
