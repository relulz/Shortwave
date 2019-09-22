use glib::Sender;
use gtk::prelude::*;

use crate::api::Station;
use crate::app::Action;
use crate::ui::station_dialog::StationDialog;

pub struct StationRow {
    pub widget: gtk::FlowBoxChild,
    station: Station,
    app: gtk::Application,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationRow {
    pub fn new(sender: Sender<Action>, station: Station) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_row.ui");
        get_widget!(builder, gtk::FlowBoxChild, station_row);
        let app = builder.get_application().unwrap();

        // Set row information
        get_widget!(builder, gtk::Label, station_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        station_label.set_text(&station.name);
        subtitle_label.set_text(&format!("{} {} · {} Votes", station.country, station.state, station.votes));

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
        get_widget!(self.builder, gtk::Button, button);
        let sender = self.sender.clone();
        button.connect_clicked(move |_| {
            let window = app.get_active_window().unwrap();
            let station_dialog = StationDialog::new(sender.clone(), station.clone(), &window);
            station_dialog.show();
        });
    }
}
