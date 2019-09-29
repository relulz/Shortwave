use glib::Sender;
use gtk::prelude::*;
use glib::futures::FutureExt;

use crate::api::{Station, FaviconDownloader};
use crate::app::Action;
use crate::ui::{StationDialog, StationFavicon, FaviconSize};

pub struct StationRow {
    pub widget: gtk::FlowBoxChild,
    station: Station,
    app: gtk::Application,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationRow {
    pub fn new(sender: Sender<Action>, favicon_downloader: FaviconDownloader, station: Station) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_row.ui");
        get_widget!(builder, gtk::FlowBoxChild, station_row);
        let app = builder.get_application().unwrap();

        // Set row information
        get_widget!(builder, gtk::Label, station_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        station_label.set_text(&station.name);
        subtitle_label.set_text(&format!("{} {} · {} Votes", station.country, station.state, station.votes));

        // Download & set station favicon
        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = StationFavicon::new(FaviconSize::Small);
        favicon_box.add(&station_favicon.widget);
        station.favicon.as_ref().map(|favicon| {
            let fut = favicon_downloader.download(favicon.clone(), FaviconSize::Small as i32).map(move|pixbuf|{
                pixbuf.ok().map(|pixbuf| station_favicon.set_pixbuf(pixbuf));
            });
            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);
        });

        let stationrow = Self {
            widget: station_row,
            station,
            app,
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
            sender.send(Action::PlaybackSetStation(station.clone())).unwrap();
        });

        // button
        let station = self.station.clone();
        let app = self.app.clone();
        get_widget!(self.builder, gtk::EventBox, eventbox);
        let sender = self.sender.clone();
        eventbox.connect_button_press_event(move |_, button| {
            // 3 -> Right mouse button
            if button.get_button() != 3 {
                let window = app.get_active_window().unwrap();
                let station_dialog = StationDialog::new(sender.clone(), station.clone(), &window);
                station_dialog.show();
            }
            gtk::Inhibit(false)
        });
    }
}
