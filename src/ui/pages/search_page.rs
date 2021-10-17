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
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{gio, glib};
use once_cell::unsync::OnceCell;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::{Notification, SwStationFlowBox};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/search_page.ui")]
    pub struct SwSearchPage {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub flowbox: TemplateChild<SwStationFlowBox>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub sorting_button_content: TemplateChild<adw::ButtonContent>,

        pub search_action_group: gio::SimpleActionGroup,

        pub station_request: Rc<RefCell<StationRequest>>,
        pub client: Client,
        pub timeout_id: Rc<RefCell<Option<glib::source::SourceId>>>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwSearchPage {
        const NAME: &'static str = "SwSearchPage";
        type ParentType = adw::Bin;
        type Type = super::SwSearchPage;

        fn new() -> Self {
            let search_action_group = gio::SimpleActionGroup::new();
            let station_request = Rc::new(RefCell::new(StationRequest::search_for_name(None, 250)));
            let client = Client::new(settings_manager::string(Key::ApiLookupDomain));
            let timeout_id = Rc::new(RefCell::new(None));

            Self {
                stack: TemplateChild::default(),
                flowbox: TemplateChild::default(),
                search_entry: TemplateChild::default(),
                sorting_button_content: TemplateChild::default(),
                search_action_group,
                station_request,
                client,
                timeout_id,
                sender: OnceCell::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwSearchPage {
        fn constructed(&self, obj: &Self::Type) {
            obj.insert_action_group("search", Some(&self.search_action_group));
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

        let model = &*imp.client.model.clone();
        imp.flowbox.init(model.clone(), sender.clone());
        imp.sender.set(sender).unwrap();

        self.setup_signals();
        self.setup_gactions();
    }

    fn setup_signals(&self) {
        let imp = imp::SwSearchPage::from_instance(self);

        imp.search_entry.connect_search_changed(clone!(@weak self as this => move |entry| {
            let imp = imp::SwSearchPage::from_instance(&this);
            let text = entry.text().to_string();

            let text = if text.is_empty() {
                None
            }else{
                Some(text)
            };

            // Update station request and redo search
            let station_request = StationRequest{
                name: text,
                ..imp.station_request.borrow().clone()
            };
            *imp.station_request.borrow_mut() = station_request;
            this.update_search();
        }));

        self.connect_map(|this| {
            let imp = imp::SwSearchPage::from_instance(&this);
            imp.search_entry.grab_focus();
            imp.search_entry.select_region(0, -1);
        });
    }

    fn setup_gactions(&self) {
        let imp = imp::SwSearchPage::from_instance(self);
        let variant_ty = Some(glib::VariantTy::new("s").unwrap());

        let sorting_action = gio::SimpleAction::new_stateful("sorting", variant_ty, &"Votes".to_variant());
        imp.search_action_group.add_action(&sorting_action);
        sorting_action.connect_change_state(clone!(@weak self as this => move |action, state|{
            let imp = imp::SwSearchPage::from_instance(&this);
            if let Some(state) = state{
                action.set_state(&state);
                let order = state.str().unwrap();

                let label = match order{
                    "Name" => i18n("Name"),
                    "Language" => i18n("Language"),
                    "Country" => i18n("Country"),
                    "State" => i18n("State"),
                    "Votes" => i18n("Votes"),
                    "Bitrate" => i18n("Bitrate"),
                    _ => panic!("unknown sorting state change"),
                };

                imp.sorting_button_content.set_label(&label);

                // Update station request and redo search
                let station_request = StationRequest{
                    order: Some(order.to_lowercase()),
                    ..imp.station_request.borrow().clone()
                };
                *imp.station_request.borrow_mut() = station_request;

                this.update_search();
            }
        }));

        let order_action = gio::SimpleAction::new_stateful("order", variant_ty, &"Descending".to_variant());
        imp.search_action_group.add_action(&order_action);
        order_action.connect_change_state(clone!(@weak self as this => move |action, state|{
            let imp = imp::SwSearchPage::from_instance(&this);
            if let Some(state) = state{
                action.set_state(&state);

                let reverse = if state.str().unwrap() == "Ascending" {
                    imp.sorting_button_content.set_icon_name("view-sort-ascending-symbolic");
                    false
                }else{
                    imp.sorting_button_content.set_icon_name("view-sort-descending-symbolic");
                    true
                };

                // Update station request and redo search
                let station_request = StationRequest{
                    reverse: Some(reverse),
                    ..imp.station_request.borrow().clone()
                };
                *imp.station_request.borrow_mut() = station_request;

                this.update_search();
            }
        }));
    }

    pub fn update_search(&self) {
        let imp = imp::SwSearchPage::from_instance(self);

        // Reset previous timeout
        let id: Option<glib::source::SourceId> = imp.timeout_id.borrow_mut().take();
        if let Some(id) = id {
            glib::source::source_remove(id)
        }

        // Don't search if search entry is empty
        if imp.station_request.borrow().name.is_none() {
            imp.stack.set_visible_child_name("empty");
            return;
        }

        imp.stack.set_visible_child_name("spinner");

        // Start new timeout
        let id = glib::source::timeout_add_seconds_local(
            1,
            clone!(@weak self as this => @default-return glib::Continue(false), move || {
                let imp = imp::SwSearchPage::from_instance(&this);
                *imp.timeout_id.borrow_mut() = None;

                let request = imp.station_request.borrow().clone();
                debug!("Search for: {:?}", request);

                let fut = imp.client.clone().send_station_request(request).map(clone!(@weak this => move |result| {
                    let imp = imp::SwSearchPage::from_instance(&this);
                    if imp.client.model.n_items() == 0{
                        imp.stack.set_visible_child_name("no-results");
                    }else{
                        imp.stack.set_visible_child_name("results");
                    }

                    if let Err(err) = result {
                        let notification = Notification::new_error(&i18n("Station data could not be received."), &err.to_string());
                        send!(imp.sender.get().unwrap(), Action::ViewShowNotification(notification));
                    }
                }));

                spawn!(fut);
                glib::Continue(false)
            }),
        );
        *imp.timeout_id.borrow_mut() = Some(id);
    }
}
