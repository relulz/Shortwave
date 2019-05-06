use glib::Sender;
use gtk::prelude::*;
use rustio::{Client, StationSearch};

use std::cell::RefCell;

use crate::app::Action;
use crate::discover::TagButton;
use crate::model::StationModel;
use crate::widgets::StationFlowBox;

pub struct StoreFront {
    pub widget: gtk::Box,
    result_model: RefCell<StationModel>,
    tags_flowbox: gtk::FlowBox,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StoreFront {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/storefront.ui");
        let widget: gtk::Box = builder.get_object("storefront").unwrap();
        let tags_flowbox: gtk::FlowBox = builder.get_object("tags_flowbox").unwrap();

        let result_model = RefCell::new(StationModel::new());
        let results_box: gtk::Box = builder.get_object("results_box").unwrap();
        let station_flowbox = StationFlowBox::new(sender.clone());
        station_flowbox.bind_model(&result_model.borrow());
        results_box.add(&station_flowbox.widget);

        let storefront = Self {
            widget,
            result_model,
            tags_flowbox,
            builder,
            sender,
        };

        storefront.add_popular_tag("Pop", "pop");
        storefront.add_popular_tag("Rock", "rock");
        storefront.add_popular_tag("Classic", "classic");
        storefront.add_popular_tag("Jazz", "jazz");

        storefront.setup_signals();
        storefront
    }

    fn add_popular_tag(&self, title: &str, name: &str) {
        let tagbutton = TagButton::new(self.sender.clone(), title, name);
        self.tags_flowbox.add(&tagbutton.widget);
    }

    pub fn search_for(&self, data: StationSearch) {
        debug!("search for: {:?}", data);
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_LOW);
        std::thread::spawn(move || {
            let mut client = Client::new("http://www.radio-browser.info");
            let result = client.search(data);
            let _ = sender.send(result);
        });
        let result_model = self.result_model.clone();
        receiver.attach(None, move |result| {
            match result {
                Ok(stations) => {
                    result_model.borrow_mut().clear();
                    for station in stations {
                        result_model.borrow_mut().add_station(station);
                    }
                }
                Err(_) => {}
            }
            glib::Continue(false)
        });
    }

    fn setup_signals(&self) {
        let search_entry: gtk::SearchEntry = self.builder.get_object("search_entry").unwrap();
        let sender = self.sender.clone();
        search_entry.connect_search_changed(move |entry| {
            let data = StationSearch::search_for_name(entry.get_text().unwrap().to_string(), false, 100);
            sender.send(Action::SearchFor(data)).unwrap();
        });
    }
}
