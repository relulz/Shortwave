// Shortwave - station.rs
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

use glib::{ObjectExt, ParamSpec, ToValue};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::api::StationMetadata;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SwStation {
        pub uuid: OnceCell<String>,
        pub is_local: OnceCell<bool>,
        pub metadata: OnceCell<StationMetadata>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStation {
        const NAME: &'static str = "SwStation";
        type ParentType = glib::Object;
        type Type = super::SwStation;
    }

    impl ObjectImpl for SwStation {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpec::new_string("uuid", "UUID", "UUID", None, glib::ParamFlags::READABLE),
                    ParamSpec::new_boolean("is-local", "Is a local station", "Is a local station", false, glib::ParamFlags::READABLE),
                    ParamSpec::new_boxed("metadata", "Metadata", "Metadata", StationMetadata::static_type(), glib::ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uuid" => self.uuid.get().unwrap().to_value(),
                "is-local" => self.is_local.get().unwrap().to_value(),
                "metadata" => self.metadata.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SwStation(ObjectSubclass<imp::SwStation>);
}

impl SwStation {
    pub fn new(uuid: String, is_local: bool, metadata: StationMetadata) -> Self {
        let station = glib::Object::new::<Self>(&[]).unwrap();

        let imp = imp::SwStation::from_instance(&station);
        imp.uuid.set(uuid).unwrap();
        imp.is_local.set(is_local).unwrap();
        imp.metadata.set(metadata).unwrap();

        station
    }

    pub fn uuid(&self) -> String {
        self.property("uuid").unwrap().get::<String>().unwrap().unwrap()
    }

    pub fn is_local(&self) -> bool {
        self.property("is-local").unwrap().get::<bool>().unwrap().unwrap()
    }

    pub fn metadata(&self) -> StationMetadata {
        self.property("metadata").unwrap().get::<&StationMetadata>().unwrap().unwrap().clone()
    }
}
