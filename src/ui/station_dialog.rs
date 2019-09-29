use glib::Sender;
use gtk::prelude::*;
use libhandy::Dialog;
use glib::futures::FutureExt;

use crate::api::{Station, FaviconDownloader};
use crate::app::Action;
use crate::database::Library;
use crate::ui::{StationFavicon, FaviconSize};

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
        get_widget!(builder, Dialog, station_dialog);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Label, codec_label);
        get_widget!(builder, gtk::Label, homepage_label);
        get_widget!(builder, gtk::Label, tags_label);
        get_widget!(builder, gtk::Label, language_label);

        // Download & set station favicon
        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = StationFavicon::new(FaviconSize::Big);
        favicon_box.add(&station_favicon.widget);
        let favicon_downloader = FaviconDownloader::new();
        let fut = favicon_downloader.download(station.favicon.clone(), FaviconSize::Big as i32).map(move|pixbuf|{
            pixbuf.ok().map(|pixbuf| station_favicon.set_pixbuf(pixbuf));
        });
        let ctx = glib::MainContext::default();
        ctx.spawn_local(fut);

        // Show correct library action
        get_widget!(builder, gtk::Stack, library_action_stack);
        if Library::contains_station(&station) {
            library_action_stack.set_visible_child_name("library-remove");
        } else {
            library_action_stack.set_visible_child_name("library-add");
        }

        station_dialog.set_transient_for(Some(window));

        let dialog = Self {
            widget: station_dialog,
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
        self.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", self.station.homepage, self.station.homepage));

        if self.station.codec != "" {
            self.codec_label.set_text(&self.station.codec);
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
        get_widget!(self.builder, gtk::Stack, library_action_stack);
        get_widget!(self.builder, gtk::Button, remove_button);
        let sender = self.sender.clone();
        let station = self.station.clone();
        remove_button.connect_clicked(move |_| {
            sender.send(Action::LibraryRemoveStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-add");
        });

        // add_button
        get_widget!(self.builder, gtk::Stack, library_action_stack);
        get_widget!(self.builder, gtk::Button, add_button);
        let sender = self.sender.clone();
        let station = self.station.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::LibraryAddStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-remove");
        });
    }
}
