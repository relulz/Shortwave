use glib::Sender;
use gtk::prelude::*;
use indexmap::IndexMap;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use crate::api::Station;
use crate::app::Action;
use crate::ui::{StationDialog, StationRow};
use crate::utils;
use crate::utils::{Order, Sorting};

pub struct StationFlowBox {
    pub widget: gtk::FlowBox,
    stations: Rc<RefCell<IndexMap<i32, Station>>>,

    sorting: RefCell<Sorting>,
    order: RefCell<Order>,

    sender: Sender<Action>,
}

impl StationFlowBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui");
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
        let stations = self.stations.clone();
        let sender = self.sender.clone();
        self.widget.connect_child_activated(move |_, child| {
            let index = child.get_index();
            let station = stations.borrow().get_index(index.try_into().unwrap()).unwrap().1.clone();

            let station_dialog = StationDialog::new(sender.clone(), station.clone());
            station_dialog.show();
        });
    }

    pub fn add_stations(&self, stations: Vec<Station>) {
        for station in stations {
            if self.stations.borrow().contains_key(&station.id) {
                warn!("Station \"{}\" is already added to flowbox.", station.name);
            } else {
                self.stations.borrow_mut().insert(station.id.clone(), station);
            }
        }

        self.sort();
        self.update_rows();
    }

    pub fn remove_stations(&self, stations: Vec<Station>) {
        for station in stations {
            // Get the corresponding widget to the index, remove and destroy it
            let index: usize = self.stations.borrow_mut().entry(station.id.clone()).index();
            dbg!(index);
            let widget = self.widget.get_child_at_index(index.try_into().unwrap()).unwrap();
            self.widget.remove(&widget);
            widget.destroy();

            // Remove the station from the indexmap itself
            self.stations.borrow_mut().shift_remove(&station.id);
        }
    }

    pub fn set_sorting(&self, sorting: Sorting, order: Order) {
        *self.sorting.borrow_mut() = sorting.clone();
        *self.order.borrow_mut() = order.clone();

        self.sort();
        self.update_rows();
    }

    // Clears everything
    pub fn clear(&self) {
        self.stations.borrow_mut().clear();
        self.clear_rows();
    }

    // Only destroy all rows, but don't clear the indexmap itself
    fn clear_rows(&self) {
        let children = self.widget.get_children();
        for widget in children {
            self.widget.remove(&widget);
            widget.destroy();
        }
    }

    fn update_rows(&self) {
        self.clear_rows();

        let widget = self.widget.downgrade();
        let sender = self.sender.clone();
        let stations = self.stations.borrow().clone();
        let constructor = move |station: (i32, Station)| StationRow::new(sender.clone(), station.1).widget.clone();

        // Start lazy loading
        utils::lazy_load(stations.clone(), widget.clone(), constructor.clone());
    }

    fn sort(&self) {
        self.stations
            .borrow_mut()
            .sort_by(move |_, a, _, b| utils::station_cmp(a, b, self.sorting.borrow().clone(), self.order.borrow().clone()));
    }
}
