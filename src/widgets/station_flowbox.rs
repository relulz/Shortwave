use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::Station;
use crate::app::Action;
use crate::model::{ModelHandler, StationModel};
use crate::utils::*;
use crate::widgets::station_row::StationRow;

pub struct StationFlowBox {
    pub widget: gtk::FlowBox,
    lazy_loading: Rc<RefCell<gio::Cancellable>>,

    sender: Sender<Action>,
}

impl StationFlowBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui");
        let widget: gtk::FlowBox = builder.get_object("station_flowbox").unwrap();
        let lazy_loading = Rc::new(RefCell::new(gio::Cancellable::new()));

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

        Self { widget, lazy_loading, sender }
    }
}

impl ModelHandler for StationFlowBox {
    fn add_stations(&self, stations: Vec<Station>) {
        let lazy_loading = self.lazy_loading.clone();
        let widget = self.widget.downgrade();
        let sender = self.sender.clone();
        let constructor = move |station| StationRow::new(sender.clone(), station).widget.clone();

        // Create new cancellable which we can use to cancel
        // the lazy loading
        let cancellable = gio::Cancellable::new();
        *lazy_loading.borrow_mut() = cancellable.clone();

        // Start lazy loading
        lazy_load(stations.clone(), widget.clone(), constructor.clone(), cancellable.clone());
    }

    fn remove_stations(&self, stations: Vec<Station>) {
        // TODO: implement remove
    }

    fn clear(&self) {
        // Cancel previous lazy loading, since we don't need
        // the content anymore because we're clearing everything.
        self.lazy_loading.borrow().cancel();

        // Fetch all children and destroy them
        let children = self.widget.get_children();
        for widget in children {
            self.widget.remove(&widget);
            widget.destroy();
        }
    }
}
