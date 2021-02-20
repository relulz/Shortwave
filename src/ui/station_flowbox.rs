// Shortwave - station_flowbox.rs
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

use glib::clone;
use glib::Sender;
use gtk::glib;
use gtk::prelude::*;
use indexmap::IndexMap;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use crate::api::SwStation;
use crate::app::Action;
use crate::ui::{StationDialog, StationRow};
use crate::utils;
use crate::utils::{Order, Sorting};

#[derive(Debug)]
pub struct StationFlowBox {
    pub widget: gtk::FlowBox,
    stations: Rc<RefCell<IndexMap<String, SwStation>>>,

    sorting: RefCell<Sorting>,
    order: RefCell<Order>,

    sender: Sender<Action>,
}

impl StationFlowBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui");
        get_widget!(builder, gtk::FlowBox, station_flowbox);
        let stations = Rc::new(RefCell::new(IndexMap::new()));

        let sorting = RefCell::new(Sorting::Default);
        let order = RefCell::new(Order::Ascending);

        let flowbox = Self {
            widget: station_flowbox,
            stations,
            sorting,
            order,
            sender,
        };

        flowbox.setup_signals();
        flowbox
    }

    fn setup_signals(&self) {
        // Show StationDialog when row gets clicked
        self.widget
            .connect_child_activated(clone!(@strong self.stations as stations, @strong self.sender as sender => move |_, child| {
                let index = child.get_index();
                let station = stations.borrow().get_index(index.try_into().unwrap()).unwrap().1.clone();

                let station_dialog = StationDialog::new(sender.clone(), station);
                station_dialog.show();
            }));
    }

    pub fn add_stations(&self, stations: Vec<SwStation>) {
        for station in stations {
            if self.stations.borrow().contains_key(&station.metadata().stationuuid) {
                warn!("SwStation \"{}\" is already added to flowbox.", station.metadata().name);
            } else {
                self.stations.borrow_mut().insert(station.metadata().stationuuid.clone(), station);
            }
        }

        self.sort();
        self.update_rows();
    }

    pub fn remove_stations(&self, stations: Vec<SwStation>) {
        for station in stations {
            // Get the corresponding widget to the index, remove and destroy it
            let index: usize = self.stations.borrow_mut().entry(station.metadata().stationuuid.clone()).index();
            let widget = self.widget.get_child_at_index(index.try_into().unwrap()).unwrap();
            self.widget.remove(&widget);

            // Remove the station from the indexmap itself
            self.stations.borrow_mut().shift_remove(&station.metadata().stationuuid);
        }
    }

    pub fn set_sorting(&self, sorting: Sorting, order: Order) {
        *self.sorting.borrow_mut() = sorting;
        *self.order.borrow_mut() = order;

        self.sort();
        self.update_rows();
    }

    // Clears everything
    pub fn clear(&self) {
        self.stations.borrow_mut().clear();
        utils::remove_all_items(&self.widget);
    }

    fn update_rows(&self) {
        utils::remove_all_items(&self.widget);

        let widget = self.widget.downgrade();
        let sender = self.sender.clone();
        let stations = self.stations.borrow().clone();
        let constructor = move |station: (String, SwStation)| StationRow::new(sender.clone(), station.1).widget;

        // Start lazy loading
        utils::lazy_load(stations, widget, constructor);
    }

    fn sort(&self) {
        self.stations
            .borrow_mut()
            .sort_by(move |_, a, _, b| utils::station_cmp(a, b, self.sorting.borrow().clone(), self.order.borrow().clone()));
    }
}
