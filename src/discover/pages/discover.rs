use glib::futures::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::settings::{Key, SettingsManager};
use crate::ui::{Notification, StationFlowBox};

pub struct Discover {
    pub widget: gtk::Box,
    client: Client,

    votes_flowbox: Rc<StationFlowBox>,
    trending_flowbox: Rc<StationFlowBox>,
    clicked_flowbox: Rc<StationFlowBox>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Discover {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/discover.ui");
        get_widget!(builder, gtk::Box, discover);

        let client = Client::new(Url::parse(&SettingsManager::get_string(Key::ApiServer)).unwrap());

        get_widget!(builder, gtk::Box, votes_box);
        let votes_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        votes_box.add(&votes_flowbox.widget);

        get_widget!(builder, gtk::Box, trending_box);
        let trending_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        trending_box.add(&trending_flowbox.widget);

        get_widget!(builder, gtk::Box, clicked_box);
        let clicked_flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        clicked_box.add(&clicked_flowbox.widget);

        let search = Self {
            widget: discover,
            client,
            votes_flowbox,
            trending_flowbox,
            clicked_flowbox,
            builder,
            sender,
        };

        search.fetch_data();
        search
    }

    fn fetch_data(&self) {
        // Stations with the most votes
        let mut votes_request = StationRequest::default();
        votes_request.order = Some("votes".to_string());
        votes_request.limit = Some(12);
        votes_request.reverse = Some(true);
        self.fill_flowbox(self.votes_flowbox.clone(), votes_request);

        // Stations with the highest clicktrend
        let mut trending_request = StationRequest::default();
        trending_request.order = Some("clicktrend".to_string());
        trending_request.limit = Some(12);
        self.fill_flowbox(self.trending_flowbox.clone(), trending_request);

        // Stations which got recently clicked
        let mut clicked_request = StationRequest::default();
        clicked_request.order = Some("clicktimestamp".to_string());
        clicked_request.limit = Some(12);
        self.fill_flowbox(self.clicked_flowbox.clone(), clicked_request);
    }

    fn fill_flowbox(&self, fb: Rc<StationFlowBox>, request: StationRequest) {
        let client = self.client.clone();
        let flowbox = fb.clone();
        let sender = self.sender.clone();
        let fut = client.send_station_request(request).map(move |stations| match stations {
            Ok(s) => {
                flowbox.clear();
                flowbox.add_stations(s);
            }
            Err(err) => {
                let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
            }
        });

        let ctx = glib::MainContext::default();
        ctx.spawn_local(fut);
    }
}
