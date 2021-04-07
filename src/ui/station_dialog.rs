// Shortwave - station_dialog.rs
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use futures_util::future::FutureExt;
use glib::clone;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{gdk, gio, glib};
use once_cell::unsync::OnceCell;

use crate::api::{FaviconDownloader, SwStation};
use crate::app::{Action, SwApplication};
use crate::database::SwLibrary;
use crate::ui::{FaviconSize, StationFavicon};
use crate::utils;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/station_dialog.ui")]
    pub struct SwStationDialog {
        #[template_child]
        pub favicon_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub library_add_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub library_remove_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub start_playback_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub language_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub language_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tags_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub tags_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub codec_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub codec_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub homepage_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub homepage_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub copy_homepage_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub stream_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stream_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub copy_stream_button: TemplateChild<gtk::Button>,

        pub station: OnceCell<SwStation>,
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwStationDialog {
        const NAME: &'static str = "SwStationDialog";
        type ParentType = adw::Window;
        type Type = super::SwStationDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwStationDialog {}

    impl WidgetImpl for SwStationDialog {}

    impl WindowImpl for SwStationDialog {}

    impl AdwWindowImpl for SwStationDialog {}
}

glib::wrapper! {
    pub struct SwStationDialog(ObjectSubclass<imp::SwStationDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl SwStationDialog {
    pub fn new(sender: Sender<Action>, station: SwStation) -> Self {
        let dialog = glib::Object::new(&[]).unwrap();

        let imp = imp::SwStationDialog::from_instance(&dialog);
        imp.station.set(station).unwrap();
        imp.sender.set(sender).unwrap();

        let window = gio::Application::get_default().unwrap().downcast_ref::<SwApplication>().unwrap().get_active_window().unwrap();
        dialog.set_transient_for(Some(&window));

        dialog.setup_widgets();
        dialog.setup_signals();
        dialog
    }

    fn setup_widgets(&self) {
        let imp = imp::SwStationDialog::from_instance(self);
        let metadata = imp.station.get().unwrap().metadata().clone();

        // Download & set station favicon
        let station_favicon = StationFavicon::new(FaviconSize::Big);
        imp.favicon_box.append(&station_favicon.widget);
        if let Some(favicon) = metadata.favicon.as_ref() {
            let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Big as i32).map(move |pixbuf| {
                if let Ok(pixbuf) = pixbuf {
                    station_favicon.set_pixbuf(pixbuf)
                }
            });
            spawn!(fut);
        }

        // Title / Subtitle
        let subtitle = utils::station_subtitle(metadata.clone());
        imp.title_label.set_text(&metadata.name);
        imp.subtitle_label.set_text(&subtitle);

        // Action rows
        if SwLibrary::contains_station(&imp.station.get().unwrap()) {
            imp.library_remove_row.set_visible(true);
        } else {
            imp.library_add_row.set_visible(true);
        }

        if !metadata.language.is_empty() {
            imp.tags_row.set_visible(true);
            imp.tags_label.set_text(&metadata.tags);
        }

        if !metadata.tags.is_empty() {
            imp.language_row.set_visible(true);
            imp.language_label.set_text(&metadata.language);
        }

        if !metadata.codec.is_empty() {
            imp.codec_row.set_visible(true);
            imp.codec_label.set_text(&metadata.codec);
        }

        if let Some(homepage) = metadata.homepage {
            imp.homepage_row.set_visible(true);
            let homepage = homepage.to_string().replace("&", "&amp;");
            imp.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", &homepage, &homepage));
        }

        if let Some(url_resolved) = metadata.url_resolved {
            imp.stream_row.set_visible(true);
            let url_resolved = url_resolved.to_string().replace("&", "&amp;");
            imp.stream_label.set_markup(&format!("<a href=\"{}\">{}</a>", &url_resolved, &url_resolved));
        }
    }

    fn setup_signals(&self) {
        let imp = imp::SwStationDialog::from_instance(self);

        imp.library_add_row.connect_activated(clone!(@weak self as this => move|_|
            let imp = imp::SwStationDialog::from_instance(&this);
            let station = imp.station.get().unwrap().clone();

            send!(imp.sender.get().unwrap(), Action::LibraryAddStations(vec![station]));
            this.hide();
            this.close();
        ));

        imp.library_remove_row.connect_activated(clone!(@weak self as this => move|_|
            let imp = imp::SwStationDialog::from_instance(&this);
            let station = imp.station.get().unwrap().clone();

            send!(imp.sender.get().unwrap(), Action::LibraryRemoveStations(vec![station]));
            this.hide();
            this.close();
        ));

        imp.start_playback_row.connect_activated(clone!(@weak self as this => move|_|
            let imp = imp::SwStationDialog::from_instance(&this);
            let station = imp.station.get().unwrap().clone();

            send!(imp.sender.get().unwrap(), Action::PlaybackSetStation(Box::new(station)));
            this.hide();
            this.close();
        ));

        imp.copy_homepage_button.connect_clicked(clone!(@weak self as this => move|_|
            let imp = imp::SwStationDialog::from_instance(&this);
            let metadata = imp.station.get().unwrap().clone().metadata();

            if let Some(homepage) = metadata.homepage {
                let display = gdk::Display::get_default().unwrap();
                let clipboard = display.get_clipboard();
                clipboard.set_text(&homepage.to_string());
            }
        ));

        imp.copy_stream_button.connect_clicked(clone!(@weak self as this => move|_|
            let imp = imp::SwStationDialog::from_instance(&this);
            let metadata = imp.station.get().unwrap().clone().metadata();

            if let Some(url_resolved) = metadata.url_resolved {
                let display = gdk::Display::get_default().unwrap();
                let clipboard = display.get_clipboard();
                clipboard.set_text(&url_resolved.to_string());
            }
        ));
    }
}
