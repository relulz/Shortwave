use glib::futures::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::ui::StationFlowBox;

pub struct Search {
    pub widget: gtk::Box,

    client: Client,
    flowbox: Rc<StationFlowBox>,
    timeout_id: Rc<RefCell<Option<glib::source::SourceId>>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Search {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/search.ui");
        let widget: gtk::Box = builder.get_object("search").unwrap();

        let client = Client::new(Url::parse("http://www.radio-browser.info/webservice/").unwrap());

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        let results_box: gtk::Box = builder.get_object("results_box").unwrap();
        results_box.add(&flowbox.widget);

        let timeout_id = Rc::new(RefCell::new(None));

        let search = Self {
            widget,
            client,
            flowbox,
            timeout_id,
            builder,
            sender,
        };

        search.setup_signals();
        search
    }

    pub fn search_for(&self, request: StationRequest) {
        // Reset previous timeout
        let id: Option<glib::source::SourceId> = self.timeout_id.borrow_mut().take();
        match id {
            Some(id) => glib::source::source_remove(id),
            None => (),
        }

        // Start new timeout
        let id = self.timeout_id.clone();
        let client = self.client.clone();
        let flowbox = self.flowbox.clone();
        let id = glib::source::timeout_add_seconds_local(1, move || {
            *id.borrow_mut() = None;

            debug!("Search for: {:?}", request);

            let client = client.clone();
            let flowbox = flowbox.clone();
            let request = request.clone();
            let fut = client.send_station_request(request).map(move |stations| {
                debug!("{:?}", stations);
                flowbox.clear();
                flowbox.add_stations(stations);
            });

            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);

            glib::Continue(false)
        });
        *self.timeout_id.borrow_mut() = Some(id);
    }

    fn setup_signals(&self) {
        let search_entry: gtk::SearchEntry = self.builder.get_object("search_entry").unwrap();
        let sender = self.sender.clone();
        search_entry.connect_search_changed(move |entry| {
            let request = StationRequest::search_for_name(&entry.get_text().unwrap(), 500);
            sender.send(Action::SearchFor(request)).unwrap();
        });
    }
}
