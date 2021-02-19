// Shortwave - search_page.rs
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
use glib::clone;
use glib::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::unsync::OnceCell;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::{Notification, StationFlowBox};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/search_page.ui")]
    pub struct SwSearchPage {
        #[template_child]
        pub results_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        pub client: Client,
        pub flowbox: OnceCell<Rc<StationFlowBox>>,
        pub timeout_id: Rc<RefCell<Option<glib::source::SourceId>>>,
        pub sender: OnceCell<Sender<Action>>,
    }

    impl ObjectSubclass for SwSearchPage {
        const NAME: &'static str = "SwSearchPage";
        type ParentType = adw::Bin;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Interfaces = ();
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::SwSearchPage;

        glib::object_subclass!();

        fn new() -> Self {
            let client = Client::new(settings_manager::get_string(Key::ApiLookupDomain));
            let timeout_id = Rc::new(RefCell::new(None));

            Self {
                results_box: TemplateChild::default(),
                search_entry: TemplateChild::default(),
                client,
                flowbox: OnceCell::default(),
                timeout_id,
                sender: OnceCell::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwSearchPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_signals();
        }
    }

    impl WidgetImpl for SwSearchPage {}

    impl BinImpl for SwSearchPage {}
}

glib::wrapper! {
    pub struct SwSearchPage(ObjectSubclass<imp::SwSearchPage>)
        @extends gtk::Widget, adw::Bin;
}

impl SwSearchPage {
    pub fn init(&self, sender: Sender<Action>) {
        let imp = imp::SwSearchPage::from_instance(self);

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));
        imp.results_box.append(&flowbox.widget);

        imp.sender.set(sender).unwrap();
        imp.flowbox.set(flowbox).unwrap();
    }

    fn setup_signals(&self) {
        let imp = imp::SwSearchPage::from_instance(self);

        imp.search_entry.connect_search_changed(clone!(@weak self as this => move |entry| {
            let request = StationRequest::search_for_name(&entry.get_text().to_string(), 250);
            this.show_station_request(request);
        }));
    }

    pub fn show_station_request(&self, request: StationRequest) {
        let imp = imp::SwSearchPage::from_instance(self);

        // Reset previous timeout
        let id: Option<glib::source::SourceId> = imp.timeout_id.borrow_mut().take();
        if let Some(id) = id {
            glib::source::source_remove(id)
        }

        // Start new timeout
        let id = imp.timeout_id.clone();
        let client = imp.client.clone();
        let flowbox = imp.flowbox.clone();
        let sender = imp.sender.get().unwrap().clone();
        let id = glib::source::timeout_add_seconds_local(1, move || {
            *id.borrow_mut() = None;

            debug!("Search for: {:?}", request);

            let client = client.clone();
            let flowbox = flowbox.clone();
            let request = request.clone();
            let sender = sender.clone();
            let fut = client.send_station_request(request).map(move |stations| match stations {
                Ok(s) => {
                    flowbox.get().unwrap().clear();
                    flowbox.get().unwrap().add_stations(s);
                }
                Err(err) => {
                    let notification = Notification::new_error(&i18n("Station data could not be received."), &err.to_string());
                    send!(sender, Action::ViewShowNotification(notification));
                }
            });

            spawn!(fut);
            glib::Continue(false)
        });
        *imp.timeout_id.borrow_mut() = Some(id);
    }
}
