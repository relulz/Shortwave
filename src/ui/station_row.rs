use futures_util::future::FutureExt;
use glib::Sender;
use gtk::prelude::*;

use crate::api::{FaviconDownloader, Station};
use crate::app::Action;
use crate::ui::{FaviconSize, StationFavicon};

pub struct StationRow {
    pub widget: gtk::FlowBoxChild,
    station: Station,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationRow {
    pub fn new(sender: Sender<Action>, station: Station) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_row.ui");
        get_widget!(builder, gtk::FlowBoxChild, station_row);

        // Set row information
        get_widget!(builder, gtk::Label, station_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        station_label.set_text(&station.name);
        subtitle_label.set_text(&format!("{} {} · {} Votes", station.country, station.state, station.votes));

        // Download & set station favicon
        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = StationFavicon::new(FaviconSize::Small);
        favicon_box.add(&station_favicon.widget);
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
        let sender = self.sender.clone();
        let station = self.station.clone();
        play_button.connect_clicked(move |_| {
            send!(sender, Action::PlaybackSetStation(Box::new(station.clone())));
        });
    }
}
