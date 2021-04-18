// Shortwave - discover_page.rs
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

use adw::subclass::prelude::*;
use futures_util::FutureExt;
use glib::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::unsync::OnceCell;

use crate::api::{Client, StationRequest};
use crate::app;
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::featured_carousel::Action;
use crate::ui::{FeaturedCarousel, Notification, SwStationFlowBox};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/discover_page.ui")]
    pub struct SwDiscoverPage {
        #[template_child]
        pub carousel_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub votes_flowbox: TemplateChild<SwStationFlowBox>,
        #[template_child]
        pub trending_flowbox: TemplateChild<SwStationFlowBox>,
        #[template_child]
        pub clicked_flowbox: TemplateChild<SwStationFlowBox>,

        pub sender: OnceCell<Sender<app::Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwDiscoverPage {
        const NAME: &'static str = "SwDiscoverPage";
        type ParentType = adw::Bin;
        type Type = super::SwDiscoverPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwDiscoverPage {}

    impl WidgetImpl for SwDiscoverPage {}

    impl BinImpl for SwDiscoverPage {}
}

glib::wrapper! {
    pub struct SwDiscoverPage(ObjectSubclass<imp::SwDiscoverPage>)
        @extends gtk::Widget, adw::Bin;
}

impl SwDiscoverPage {
    pub fn init(&self, sender: Sender<app::Action>) {
        let imp = imp::SwDiscoverPage::from_instance(self);
        imp.sender.set(sender).unwrap();

        self.setup_widgets();
    }

    fn setup_widgets(&self) {
        let imp = imp::SwDiscoverPage::from_instance(self);

        // Featured Carousel
        let carousel = FeaturedCarousel::new();
        imp.carousel_box.append(&carousel.widget);

        let _action = Action::new("win.show-server-stats", &i18n("Show statistics"));
        carousel.add_page(&i18n("Browse over 25,500 stations"), "26,95,180", None);

        let action = Action::new("win.create-new-station", &i18n("Add new station"));
        carousel.add_page(&i18n("Your favorite station is missing?"), "229,165,10", Some(action));

        let action = Action::new("win.open-radio-browser-info", &i18n("Open website"));
        carousel.add_page(&i18n("Powered by radio-browser.info"), "38,162,105", Some(action));

        // Most voted stations (stations with the most votes)
        let votes_request = StationRequest {
            order: Some("votes".to_string()),
            limit: Some(12),
            reverse: Some(true),
            ..Default::default()
        };
        self.fill_flowbox(&imp.votes_flowbox, votes_request);

        // Trending (stations with the highest clicktrend)
        let trending_request = StationRequest {
            order: Some("clicktrend".to_string()),
            limit: Some(12),
            ..Default::default()
        };
        self.fill_flowbox(&imp.trending_flowbox, trending_request);

        // Other users are listening to... (stations which got recently clicked)
        let clicked_request = StationRequest {
            order: Some("clicktimestamp".to_string()),
            limit: Some(12),
            ..Default::default()
        };
        self.fill_flowbox(&imp.clicked_flowbox, clicked_request);
    }

    fn fill_flowbox(&self, flowbox: &SwStationFlowBox, request: StationRequest) {
        let imp = imp::SwDiscoverPage::from_instance(self);

        let client = Client::new(settings_manager::string(Key::ApiLookupDomain));
        let sender = imp.sender.get().unwrap().clone();

        let model = &*client.model;
        flowbox.init(model.clone(), sender.clone());

        let fut = client.send_station_request(request).map(move |result| {
            if let Err(err) = result {
                let notification = Notification::new_error(&i18n("Station data could not be received."), &err.to_string());
                send!(sender, app::Action::ViewShowNotification(notification));
            }
        });

        spawn!(fut);
    }
}
