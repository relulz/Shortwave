// Shortwave - station_flowbox.rs
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
use glib::clone;
use glib::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::unsync::OnceCell;

use crate::api::SwStation;
use crate::app::Action;
use crate::model::SwStationModel;
use crate::ui::{StationDialog, SwStationRow};
use crate::utils;
use crate::utils::{Order, Sorting};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui")]
    pub struct SwStationFlowBox {
        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,
        pub model: OnceCell<gtk::SortListModel>,

        pub sender: OnceCell<Sender<Action>>,
    }

    impl ObjectSubclass for SwStationFlowBox {
        const NAME: &'static str = "SwStationFlowBox";
        type ParentType = adw::Bin;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Interfaces = ();
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::SwStationFlowBox;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                flowbox: TemplateChild::default(),
                model: OnceCell::default(),
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

    impl ObjectImpl for SwStationFlowBox {}

    impl WidgetImpl for SwStationFlowBox {}

    impl BinImpl for SwStationFlowBox {}
}

glib::wrapper! {
    pub struct SwStationFlowBox(ObjectSubclass<imp::SwStationFlowBox>)
        @extends gtk::Widget, adw::Bin;
}

impl SwStationFlowBox {
    pub fn init(&self, model: SwStationModel, sender: Sender<Action>) {
        let imp = imp::SwStationFlowBox::from_instance(self);
        imp.sender.set(sender.clone()).unwrap();

        //let sorter = gtk::StringSorter::
        //let sortlist_model = gtk::SortListModel::new(model, sorter);

        imp.flowbox.get().bind_model(
            Some(&model),
            clone!(@strong sender => move |station|{
                let station = station.downcast_ref::<SwStation>().unwrap();
                let row = SwStationRow::new(sender.clone(), station.clone());
                row.upcast()
            }),
        );

        self.setup_widgets();
        self.setup_signals();
    }

    fn setup_signals(&self) {
        let imp = imp::SwStationFlowBox::from_instance(self);

        // Show StationDialog when row gets clicked
        imp.flowbox.connect_child_activated(clone!(@strong imp.sender as sender => move |_, child| {
            let row = child.clone().downcast::<SwStationRow>().unwrap();
            let station = row.station();

            let station_dialog = StationDialog::new(sender.get().unwrap().clone(), station);
            station_dialog.show();
        }));
    }

    fn setup_widgets(&self) {
        let _imp = imp::SwStationFlowBox::from_instance(self);
    }
}
