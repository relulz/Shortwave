// Shortwave - search.rs
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

use futures_util::future::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::settings::{settings_manager, Key};
use crate::ui::{Notification, StationFlowBox};

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
        get_widget!(builder, gtk::Box, search);

        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());

        get_widget!(builder, gtk::Box, results_box);
        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        results_box.add(&flowbox.widget);

        // keyboard focus
        get_widget!(builder, gtk::ScrolledWindow, scrolledwindow);
        search.set_focus_vadjustment(&scrolledwindow.get_vadjustment().unwrap());

        let timeout_id = Rc::new(RefCell::new(None));

        let search = Self {
            widget: search,
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
        if let Some(id) = id {
            glib::source::source_remove(id)
        }

        // Start new timeout
        let id = self.timeout_id.clone();
        let client = self.client.clone();
        let flowbox = self.flowbox.clone();
        let sender = self.sender.clone();
        let id = glib::source::timeout_add_seconds_local(1, move || {
            *id.borrow_mut() = None;

            debug!("Search for: {:?}", request);

            let client = client.clone();
            let flowbox = flowbox.clone();
            let request = request.clone();
            let sender = sender.clone();
            let fut = client.send_station_request(request).map(move |stations| match stations {
                Ok(s) => {
                    flowbox.clear();
                    flowbox.add_stations(s);
                }
                Err(err) => {
                    let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                    send!(sender, Action::ViewShowNotification(notification));
                }
            });

            spawn!(fut);
            glib::Continue(false)
        });
        *self.timeout_id.borrow_mut() = Some(id);
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::SearchEntry, search_entry);
        search_entry.connect_search_changed(clone!(@strong self.sender as sender => move |entry| {
            let request = StationRequest::search_for_name(&entry.get_text().unwrap(), 250);
            send!(sender, Action::SearchFor(request));
        }));
    }
}
