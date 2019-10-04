use glib::futures::FutureExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Client, StationRequest};
use crate::app::Action;
use crate::ui::{StationFlowBox, Notification};
use crate::settings::{Key, SettingsManager};

pub struct Discover {
    pub widget: gtk::Box,

    client: Client,
    flowbox: Rc<StationFlowBox>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Discover {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/discover.ui");
        get_widget!(builder, gtk::Box, discover);

        let client = Client::new(Url::parse(&SettingsManager::get_string(Key::ApiServer)).unwrap());

        let flowbox = Rc::new(StationFlowBox::new(sender.clone()));

        let search = Self {
            widget: discover,
            client,
            flowbox,
            builder,
            sender,
        };

        search.setup_signals();
        search
    }

    fn setup_signals(&self) {

    }
}
