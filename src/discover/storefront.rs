// Shortwave - storefront.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::Sender;
use gtk::prelude::*;

use crate::api::StationRequest;
use crate::app::Action;
use crate::discover::pages::{Discover, Search};

#[allow(dead_code)]
pub struct StoreFront {
    pub widget: gtk::Box,
    pub storefront_stack: gtk::Stack,

    discover: Discover,
    search: Search,

    builder: gtk::Builder,
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
        let search = Search::new(sender);
        search_box.add(&search.widget);

        let storefront = Self {
            widget: storefront,
            storefront_stack,
            discover,
            search,
            builder,
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
