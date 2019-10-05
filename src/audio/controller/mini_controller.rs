use glib::Sender;
use glib::futures::FutureExt;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Station, FaviconDownloader};
use crate::app::Action;
use crate::audio::Controller;
use crate::audio::PlaybackState;
use crate::ui::{StationFavicon, FaviconSize};

pub struct MiniController {
    pub widget: gtk::Box,
    sender: Sender<Action>,
    station: Rc<RefCell<Option<Station>>>,

    station_favicon: Rc<StationFavicon>,
    favicon_downloader: FaviconDownloader,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    action_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    eventbox: gtk::EventBox,
}

impl MiniController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/mini_controller.ui");
        get_widget!(builder, gtk::Box, mini_controller);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Revealer, subtitle_revealer);
        get_widget!(builder, gtk::Revealer, action_revealer);
        get_widget!(builder, gtk::Stack, playback_button_stack);
        get_widget!(builder, gtk::Button, start_playback_button);
        get_widget!(builder, gtk::Button, stop_playback_button);
        get_widget!(builder, gtk::EventBox, eventbox);

        let station = Rc::new(RefCell::new(None));

        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = Rc::new(StationFavicon::new(FaviconSize::Mini));
        favicon_box.add(&station_favicon.widget);
        let favicon_downloader = FaviconDownloader::new();

        let controller = Self {
            widget: mini_controller,
            sender,
            station,
            station_favicon,
            favicon_downloader,
            title_label,
            subtitle_label,
            action_revealer,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            eventbox,
        };

        controller.setup_signals();
        controller
    }

    fn setup_signals(&self) {
        // start_playback_button
        let sender = self.sender.clone();
        self.start_playback_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackStart).unwrap();
        });

        // stop_playback_button
        let sender = self.sender.clone();
        self.stop_playback_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackStop).unwrap();
        });

        // show_player_button
        let sender = self.sender.clone();
        self.eventbox.connect_button_release_event(move |_,_| {
            sender.send(Action::ViewShowPlayer).unwrap();
            glib::signal::Inhibit(false)
        });
    }
}

impl Controller for MiniController {
    fn set_station(&self, station: Station) {
        self.action_revealer.set_reveal_child(true);
        self.title_label.set_text(&station.name);
        self.title_label.set_tooltip_text(Some(station.name.as_str()));
        *self.station.borrow_mut() = Some(station.clone());

        // Download & set icon
        let station_favicon = self.station_favicon.clone();
        station.favicon.map(|favicon| {
            let fut = self.favicon_downloader.clone().download (favicon, FaviconSize::Mini as i32).map(move|pixbuf|{
                pixbuf.ok().map(|pixbuf| station_favicon.set_pixbuf(pixbuf));
            });
            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);
        });

        // reset everything else
        self.station_favicon.reset();
        self.subtitle_revealer.set_reveal_child(false);
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        match playback_state {
            PlaybackState::Playing => self.playback_button_stack.set_visible_child_name("stop_playback"),
            PlaybackState::Stopped => self.playback_button_stack.set_visible_child_name("start_playback"),
            PlaybackState::Loading => self.playback_button_stack.set_visible_child_name("loading"),
            PlaybackState::Failure(_) => self.playback_button_stack.set_visible_child_name("start_playback"),
        };
    }

    fn set_volume(&self, _volume: f64) {
        // We don't have to do anything here.
    }

    fn set_song_title(&self, title: &str) {
        if title != "" {
            self.subtitle_label.set_text(title);
            self.subtitle_label.set_tooltip_text(Some(title));
            self.subtitle_revealer.set_reveal_child(true);
        } else {
            self.subtitle_label.set_text("");
            self.subtitle_label.set_tooltip_text(None);
            self.subtitle_revealer.set_reveal_child(false);
        }
    }
}
