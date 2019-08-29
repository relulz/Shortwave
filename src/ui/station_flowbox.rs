use glib::Sender;
use gtk::prelude::*;

use crate::api::Station;
use crate::app::Action;
use crate::ui::station_row::StationRow;
use crate::utils;

pub struct StationFlowBox {
    pub widget: gtk::FlowBox,

    sender: Sender<Action>,
}

impl StationFlowBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui");
        let widget: gtk::FlowBox = builder.get_object("station_flowbox").unwrap();

        // Set automatically flowbox colums
        let fb = widget.clone();
        widget.connect_size_allocate(move |_, alloc| {
            if alloc.width > 1000 {
                fb.set_min_children_per_line(3);
                fb.set_max_children_per_line(3);
            } else if alloc.width > 650 {
                fb.set_min_children_per_line(2);
                fb.set_max_children_per_line(2);
            } else {
                fb.set_min_children_per_line(1);
                fb.set_max_children_per_line(1);
            }
        });

        Self { widget, sender }
    }

    pub fn set_stations(&self, stations: Vec<Station>) {
        self.clear();

        let widget = self.widget.downgrade();
        let sender = self.sender.clone();
        let constructor = move |station: Station| StationRow::new(sender.clone(), station).widget.clone();

        // Start lazy loading
        utils::lazy_load(stations.clone(), widget.clone(), constructor.clone());
    }

    // Clears everything
    fn clear(&self) {
        let children = self.widget.get_children();
        for widget in children {
            self.widget.remove(&widget);
            widget.destroy();
        }
    }
}
