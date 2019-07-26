use glib::Sender;
use gtk::prelude::*;
use url::Url;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::discover::TileButton;
use crate::widgets::StationFlowBox;

pub struct StoreFront {
    pub widget: gtk::Box,
    pub discover_stack: gtk::Stack,

    tags_flowbox: gtk::FlowBox,
    client: Client,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StoreFront {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/storefront.ui");
        let widget: gtk::Box = builder.get_object("storefront").unwrap();
        let discover_stack: gtk::Stack = builder.get_object("discover_stack").unwrap();

        let tags_flowbox: gtk::FlowBox = builder.get_object("tags_flowbox").unwrap();
        let client = Client::new(Url::parse("http://www.radio-browser.info/webservice/").unwrap());

        let results_box: gtk::Box = builder.get_object("results_box").unwrap();
        let station_flowbox = StationFlowBox::new(sender.clone());
        station_flowbox.bind_model(&client.model.borrow());
        results_box.add(&station_flowbox.widget);

        let storefront = Self {
            widget,
            discover_stack,
            tags_flowbox,
            client,
            builder,
            sender,
        };

        storefront.add_popular_tag("Pop", "tags/pop");
        storefront.add_popular_tag("Rock", "tags/rock");
        storefront.add_popular_tag("Classic", "tags/classic");
        storefront.add_popular_tag("Jazz", "tags/jazz");

        storefront.setup_signals();
        storefront
    }

    fn add_popular_tag(&self, title: &str, name: &str) {
        let tagbutton = TileButton::new(self.sender.clone(), title, name);
        self.tags_flowbox.add(&tagbutton.widget);
    }

    pub fn search_for(&self, request: StationRequest) {
        debug!("Search for: {:?}", request);
        self.client.send_station_request(&request);
    }

    fn setup_signals(&self) {
        let search_entry: gtk::SearchEntry = self.builder.get_object("search_entry").unwrap();
        let sender = self.sender.clone();
        search_entry.connect_search_changed(move |entry| {
            let request = StationRequest::search_for_name(&entry.get_text().unwrap(), 200);
            sender.send(Action::SearchFor(request)).unwrap();
        });
    }
}
