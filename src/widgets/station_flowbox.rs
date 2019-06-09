use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::model::ObjectWrapper;
use crate::model::StationModel;
use crate::widgets::station_row::StationRow;

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

    pub fn bind_model(&self, model: &StationModel) {
        let sender = self.sender.clone();

        self.widget.bind_model(Some(&model.model), move |station| {
            let row = StationRow::new(sender.clone(), station.downcast_ref::<ObjectWrapper>().unwrap().deserialize());
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
