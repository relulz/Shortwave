use glib::Sender;
use gtk::prelude::*;

use crate::api::StationRequest;
use crate::app::Action;
use crate::discover::pages::Search;
use crate::discover::TileButton;

pub struct StoreFront {
    pub widget: gtk::Box,
    pub discover_stack: gtk::Stack,

    search: Search,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StoreFront {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/storefront.ui");
        let widget: gtk::Box = builder.get_object("storefront").unwrap();
        let discover_stack: gtk::Stack = builder.get_object("discover_stack").unwrap();

        let search = Search::new(sender.clone());
        let search_box: gtk::Box = builder.get_object("search_box").unwrap();
        search_box.add(&search.widget);

        let storefront = Self {
            widget,
            discover_stack,
            search,
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
        let tags_flowbox: gtk::FlowBox = self.builder.get_object("tags_flowbox").unwrap();
        let tagbutton = TileButton::new(self.sender.clone(), title, name);
        tags_flowbox.add(&tagbutton.widget);
    }

    pub fn search_for(&self, request: StationRequest) {
        self.search.search_for(request);
    }

    fn setup_signals(&self) {}
}
