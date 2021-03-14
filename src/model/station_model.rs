// Shortwave - station_model.rs
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

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::api::SwStation;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SwStationModel {
        pub vec: RefCell<Vec<SwStation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationModel {
        const NAME: &'static str = "SwStationModel";
        type ParentType = glib::Object;
        type Type = super::SwStationModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SwStationModel {}

    impl ListModelImpl for SwStationModel {
        fn get_item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SwStation::static_type()
        }
        fn get_n_items(&self, _list_model: &Self::Type) -> u32 {
            self.vec.borrow().len() as u32
        }
        fn get_item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.vec.borrow().get(position as usize).map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SwStationModel(ObjectSubclass<imp::SwStationModel>) @implements gio::ListModel;
}

impl SwStationModel {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_station(&self, station: &SwStation) {
        let imp = imp::SwStationModel::from_instance(self);

        if self.find(station).is_some() {
            warn!("Station {:?} already exists in model", station.metadata().name);
            return;
        }

        // Own scope to avoid "already mutably borrowed: BorrowError"
        let pos = {
            let mut data = imp.vec.borrow_mut();
            data.push(station.clone());
            (data.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn remove_station(&self, station: &SwStation) {
        let imp = imp::SwStationModel::from_instance(self);

        match self.find(station) {
            Some(pos) => {
                imp.vec.borrow_mut().remove(pos as usize);
                self.items_changed(pos, 1, 0);
            }
            None => warn!("Station {:?} not found in model", station.metadata().name),
        }
    }

    pub fn find(&self, station: &SwStation) -> Option<u32> {
        for pos in 0..self.get_n_items() {
            let obj = self.get_object(pos)?;
            let s = obj.downcast::<SwStation>().unwrap();
            if station.metadata().stationuuid == s.metadata().stationuuid {
                return Some(pos);
            }
        }
        None
    }

    pub fn clear(&self) {
        let imp = imp::SwStationModel::from_instance(self);
        let len = self.get_n_items();
        imp.vec.borrow_mut().clear();
        self.items_changed(0, len, 0);
    }
}
