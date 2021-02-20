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
    use glib::subclass;

    #[derive(Debug)]
    pub struct SwStation {
        pub metadata: OnceCell<StationMetadata>,
    }

    impl ObjectSubclass for SwStation {
        const NAME: &'static str = "SwStation";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Interfaces = ();
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::SwStation;

        glib::object_subclass!();

        fn new() -> Self {
            Self { metadata: OnceCell::default() }
        }
    }

    impl ObjectImpl for SwStation {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![ParamSpec::boxed("metadata", "Metadata", "Metadata", StationMetadata::static_type(), glib::ParamFlags::READABLE)]);
            PROPERTIES.as_ref()
        }

        fn get_property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.get_name() {
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
    pub fn new(metadata: StationMetadata) -> Self {
        let station = glib::Object::new::<Self>(&[]).unwrap();

        let imp = imp::SwStation::from_instance(&station);
        imp.metadata.set(metadata).unwrap();

        station
    }

    pub fn metadata(&self) -> StationMetadata {
        self.get_property("metadata").unwrap().get::<&StationMetadata>().unwrap().unwrap().clone()
    }
}
