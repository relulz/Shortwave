use glib::Sender;
use gtk::prelude::*;
use libhandy::Dialog;

use crate::api::Station;
use crate::app::Action;
use crate::database::Library;

pub struct StationDialog {
    pub widget: Dialog,
    station: Station,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    codec_label: gtk::Label,
    homepage_label: gtk::Label,
    tags_label: gtk::Label,
    language_label: gtk::Label,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationDialog {
    pub fn new(sender: Sender<Action>, station: Station, window: &gtk::Window) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_dialog.ui");
        let widget: Dialog = get_widget!(builder, "station_dialog");

        let title_label: gtk::Label = get_widget!(builder, "title_label");
        let subtitle_label: gtk::Label = get_widget!(builder, "subtitle_label");
        let codec_label: gtk::Label = get_widget!(builder, "codec_label");
        let homepage_label: gtk::Label = get_widget!(builder, "homepage_label");
        let tags_label: gtk::Label = get_widget!(builder, "tags_label");
        let language_label: gtk::Label = get_widget!(builder, "language_label");

        // Show correct library action
        let library_action_stack: gtk::Stack = get_widget!(builder, "library_action_stack");
        if Library::contains_station(&station) {
            library_action_stack.set_visible_child_name("library-remove");
        } else {
            library_action_stack.set_visible_child_name("library-add");
        }

        widget.set_transient_for(Some(window));

        let dialog = Self {
            widget,
            station,
            title_label,
            subtitle_label,
            codec_label,
            homepage_label,
            tags_label,
            language_label,
            builder,
            sender,
        };

        dialog.setup();
        dialog.setup_signals();
        dialog
    }

    fn setup(&self) {
        self.title_label.set_text(&self.station.name);
        let subtitle_text = &format!("{} {} · {} Votes", self.station.country, self.station.state, self.station.votes);
        self.subtitle_label.set_text(subtitle_text);

        if self.station.codec != "" {
            self.codec_label.set_text(&self.station.codec);
        }
        if self.station.homepage != "" {
            self.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", self.station.homepage, self.station.homepage));
        }
        if self.station.tags != "" {
            self.tags_label.set_text(&self.station.tags);
        }
        if self.station.language != "" {
            self.language_label.set_text(&self.station.language);
        }
    }

    pub fn show(&self) {
        self.widget.set_visible(true);
    }

    fn setup_signals(&self) {
        // remove_button
        let library_action_stack: gtk::Stack = get_widget!(self.builder, "library_action_stack");
        let remove_button: gtk::Button = get_widget!(self.builder, "remove_button");
        let sender = self.sender.clone();
        let station = self.station.clone();
        remove_button.connect_clicked(move |_| {
            sender.send(Action::LibraryRemoveStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-add");
        });

        // add_button
        let library_action_stack: gtk::Stack = get_widget!(self.builder, "library_action_stack");
        let add_button: gtk::Button = get_widget!(self.builder, "add_button");
        let sender = self.sender.clone();
        let station = self.station.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::LibraryAddStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-remove");
        });
    }
}
