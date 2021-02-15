// Shortwave - station_row.rs
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

use futures_util::future::FutureExt;
use glib::Sender;
use gtk::prelude::*;

use crate::api::{FaviconDownloader, Station};
use crate::app::Action;
use crate::ui::{FaviconSize, StationFavicon};
use crate::utils;

pub struct StationRow {
    pub widget: gtk::FlowBoxChild,
    station: Station,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationRow {
    pub fn new(sender: Sender<Action>, station: Station) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/station_row.ui");
        get_widget!(builder, gtk::FlowBoxChild, station_row);

        // Set row information
        get_widget!(builder, gtk::Label, station_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        station_label.set_text(&station.name);
        let subtitle = utils::station_subtitle(&station.country, &station.state, station.votes);
        subtitle_label.set_text(&subtitle);

        // Download & set station favicon
        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = StationFavicon::new(FaviconSize::Small);
        favicon_box.append(&station_favicon.widget);
        if let Some(favicon) = station.favicon.as_ref() {
            let fut = FaviconDownloader::download(favicon.clone(), FaviconSize::Small as i32).map(move |pixbuf| {
                if let Ok(pixbuf) = pixbuf {
                    station_favicon.set_pixbuf(pixbuf)
                }
            });
            spawn!(fut);
        }

        let stationrow = Self {
            widget: station_row,
            station,
            builder,
            sender,
        };

        stationrow.setup_signals();
        stationrow
    }

    fn setup_signals(&self) {
        // play_button
        get_widget!(self.builder, gtk::Button, play_button);
        play_button.connect_clicked(clone!(@strong self.sender as sender, @strong self.station as station => move |_| {
            send!(sender, Action::PlaybackSetStation(Box::new(station.clone())));
        }));
    }
}
