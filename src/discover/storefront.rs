use glib::Sender;
use gtk::prelude::*;

use crate::api::StationRequest;
use crate::app::Action;
use crate::discover::pages::{Discover, Search};

pub struct StoreFront {
    pub widget: gtk::Box,
    pub storefront_stack: gtk::Stack,

    discover: Discover,
    search: Search,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StoreFront {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/storefront.ui");
        get_widget!(builder, gtk::Box, storefront);
        get_widget!(builder, gtk::Stack, storefront_stack);

        // Discover
        get_widget!(builder, gtk::Box, discover_box);
        let discover = Discover::new(sender.clone());
        discover_box.add(&discover.widget);

        // Search
        get_widget!(builder, gtk::Box, search_box);
        let search = Search::new(sender.clone());
        search_box.add(&search.widget);

        let storefront = Self {
            widget: storefront,
            storefront_stack,
            discover,
            search,
            builder,
            sender,
        };

        storefront.setup_signals();
        storefront
    }

    pub fn search_for(&self, request: StationRequest) {
        self.storefront_stack.set_visible_child_name("search");
        self.search.search_for(request);
    }

    fn setup_signals(&self) {}
}
