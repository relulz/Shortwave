// Shortwave - discover.rs
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

use futures_util::future::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::featured_carousel;
use crate::ui::{FeaturedCarousel, Notification, StationFlowBox};

#[allow(dead_code)]
pub struct Discover {
    pub widget: gtk::Box,
    client: Client,

    carousel: FeaturedCarousel,
    votes_flowbox: Rc<StationFlowBox>,
    trending_flowbox: Rc<StationFlowBox>,
    clicked_flowbox: Rc<StationFlowBox>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Discover {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/discover.ui");
        get_widget!(builder, gtk::Box, discover);

        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());

        // Featured Carousel
        let carousel = FeaturedCarousel::new();
        get_widget!(builder, gtk::Box, carousel_box);
        carousel_box.append(&carousel.widget);

        let _action = featured_carousel::Action::new("win.show-server-stats", &i18n("Show statistics"));
        carousel.add_page(&i18n("Browse over 25,500 stations"), "26,95,180", None);

        let action = featured_carousel::Action::new("win.create-new-station", &i18n("Add new station"));
        carousel.add_page(&i18n("Your favorite station is missing?"), "229,165,10", Some(action));

        let action = featured_carousel::Action::new("win.open-radio-browser-info", &i18n("Open website"));
        carousel.add_page(&i18n("Powered by radio-browser.info"), "38,162,105", Some(action));

        // Most voted stations
        get_widget!(builder, gtk::Box, votes_box);
        let votes_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        votes_box.append(&votes_flowbox.widget);

        // Trending
        get_widget!(builder, gtk::Box, trending_box);
        let trending_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        trending_box.append(&trending_flowbox.widget);

        // Other users are listening to...
        get_widget!(builder, gtk::Box, clicked_box);
        let clicked_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        clicked_box.append(&clicked_flowbox.widget);

        let discover = Self {
            widget: discover,
            client,
            carousel,
            votes_flowbox,
            trending_flowbox,
            clicked_flowbox,
            builder,
            sender,
        };

        discover.fetch_data();
        discover
    }

    fn fetch_data(&self) {
        // Most voted stations (stations with the most votes)
        let mut votes_request = StationRequest::default();
        votes_request.order = Some("votes".to_string());
        votes_request.limit = Some(12);
        votes_request.reverse = Some(true);
        self.fill_flowbox(self.votes_flowbox.clone(), votes_request);

        // Trending (stations with the highest clicktrend)
        let mut trending_request = StationRequest::default();
        trending_request.order = Some("clicktrend".to_string());
        trending_request.limit = Some(12);
        self.fill_flowbox(self.trending_flowbox.clone(), trending_request);

        // Other users are listening to... (stations which got recently clicked)
        let mut clicked_request = StationRequest::default();
        clicked_request.order = Some("clicktimestamp".to_string());
        clicked_request.limit = Some(12);
        self.fill_flowbox(self.clicked_flowbox.clone(), clicked_request);
    }

    fn fill_flowbox(&self, fb: Rc<StationFlowBox>, request: StationRequest) {
        let client = self.client.clone();
        let flowbox = fb;
        let sender = self.sender.clone();
        let fut = client.send_station_request(request).map(move |stations| match stations {
            Ok(s) => {
                flowbox.clear();
                flowbox.add_stations(s);
            }
            Err(err) => {
                let notification = Notification::new_error(&i18n("Station data could not be received."), &err.to_string());
                send!(sender, Action::ViewShowNotification(notification));
            }
        });

        spawn!(fut);
    }
}
