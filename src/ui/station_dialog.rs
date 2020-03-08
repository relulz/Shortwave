// Shortwave - station_dialog.rs
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
use libhandy::Dialog;

use crate::api::{FaviconDownloader, Station};
use crate::app::Action;
use crate::database::Library;
use crate::ui::{FaviconSize, StationFavicon};
use crate::utils;

pub struct StationDialog {
    pub widget: Dialog,
    station: Station,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    codec_label: gtk::Label,
    homepage_label: gtk::Label,
    tags_label: gtk::Label,
    language_label: gtk::Label,

    codec_label_label: gtk::Label,
    homepage_label_label: gtk::Label,
    tags_label_label: gtk::Label,
    language_label_label: gtk::Label,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationDialog {
    pub fn new(sender: Sender<Action>, station: Station) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_dialog.ui");
        get_widget!(builder, Dialog, station_dialog);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Label, codec_label);
        get_widget!(builder, gtk::Label, homepage_label);
        get_widget!(builder, gtk::Label, tags_label);
        get_widget!(builder, gtk::Label, language_label);
        get_widget!(builder, gtk::Label, codec_label_label);
        get_widget!(builder, gtk::Label, homepage_label_label);
        get_widget!(builder, gtk::Label, tags_label_label);
        get_widget!(builder, gtk::Label, language_label_label);

        // Download & set station favicon
        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = StationFavicon::new(FaviconSize::Big);
        favicon_box.add(&station_favicon.widget);
        if let Some(favicon) = station.favicon.as_ref() {
            let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Big as i32).map(move |pixbuf| {
                if let Ok(pixbuf) = pixbuf {
                    station_favicon.set_pixbuf(pixbuf)
                }
            });
            spawn!(fut);
        }

        // Show correct library action
        get_widget!(builder, gtk::Stack, library_action_stack);
        if Library::contains_station(&station) {
            library_action_stack.set_visible_child_name("library-remove");
        } else {
            library_action_stack.set_visible_child_name("library-add");
        }

        let dialog = Self {
            widget: station_dialog,
            station,
            title_label,
            subtitle_label,
            codec_label,
            homepage_label,
            tags_label,
            language_label,
            codec_label_label,
            homepage_label_label,
            tags_label_label,
            language_label_label,
            builder,
            sender,
        };

        dialog.setup();
        dialog.setup_signals();
        dialog
    }

    fn setup(&self) {
        self.title_label.set_text(&self.station.name);
        let subtitle = utils::station_subtitle(&self.station.country, &self.station.state, self.station.votes);
        self.subtitle_label.set_text(&subtitle);

        if self.station.codec != "" {
            self.codec_label.set_text(&self.station.codec);
        } else {
            self.codec_label.hide();
            self.codec_label_label.hide();
        }
        if self.station.tags != "" {
            self.tags_label.set_text(&self.station.tags);
        } else {
            self.tags_label.hide();
            self.tags_label_label.hide();
        }
        if self.station.language != "" {
            self.language_label.set_text(&self.station.language);
        } else {
            self.language_label.hide();
            self.language_label_label.hide();
        }
        if let Some(ref homepage) = self.station.homepage {
            self.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", homepage, homepage));
        } else {
            self.homepage_label.hide();
            self.homepage_label_label.hide();
        }
    }

    pub fn show(&self) {
        let application = self.builder.get_application().unwrap();
        let window = application.get_active_window().unwrap();
        self.widget.set_transient_for(Some(&window));
        self.widget.set_visible(true);
    }

    fn setup_signals(&self) {
        // remove_button
        get_widget!(self.builder, gtk::Button, remove_button);
        remove_button.connect_clicked(clone!(@weak self.widget as widget, @strong self.station as station, @strong self.sender as sender => move |_| {
            send!(sender, Action::LibraryRemoveStations(vec![station.clone()]));
            widget.hide();
            widget.destroy();
        }));

        // add_button
        get_widget!(self.builder, gtk::Button, add_button);
        add_button.connect_clicked(clone!(@weak self.widget as widget, @strong self.station as station, @strong self.sender as sender => move |_| {
            send!(sender, Action::LibraryAddStations(vec![station.clone()]));
            widget.hide();
            widget.destroy();
        }));
    }
}
