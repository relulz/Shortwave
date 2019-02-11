use gtk::prelude::*;
use rustio::Station;

use std::sync::mpsc::Sender;

use crate::app::Action;
use crate::player::Controller;
use crate::player::PlaybackState;

pub struct GtkController {
    pub widget: gtk::Box,
    sender: Sender<Action>,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    volume_button: gtk::VolumeButton,
}

impl GtkController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/gtk_controller.ui");

        let widget: gtk::Box = builder.get_object("gtk_controller").unwrap();
        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        let subtitle_label: gtk::Label = builder.get_object("subtitle_label").unwrap();
        let subtitle_revealer: gtk::Revealer = builder.get_object("subtitle_revealer").unwrap();
        let playback_button_stack: gtk::Stack = builder.get_object("playback_button_stack").unwrap();
        let start_playback_button: gtk::Button = builder.get_object("start_playback_button").unwrap();
        let stop_playback_button: gtk::Button = builder.get_object("stop_playback_button").unwrap();
        let volume_button: gtk::VolumeButton = builder.get_object("volume_button").unwrap();

        let controller = Self {
            widget,
            sender,
            title_label,
            subtitle_label,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            volume_button,
        };

        controller.connect_signals();
        controller
    }

    fn connect_signals(&self) {
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

        // volume button
        let sender = self.sender.clone();
        self.volume_button.connect_value_changed(move |_, value| {
            sender.send(Action::PlaybackSetVolume(value)).unwrap();
        });
    }
}

impl Controller for GtkController {
    fn set_station(&self, station: Station) {
        self.title_label.set_text(&station.name);
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        match playback_state {
            PlaybackState::Playing => self.playback_button_stack.set_visible_child_name("stop_playback"),
            PlaybackState::Stopped => self.playback_button_stack.set_visible_child_name("start_playback"),
            PlaybackState::Loading => self.playback_button_stack.set_visible_child_name("loading"),
            _ => (),
        };
    }

    fn set_volume(&self, volume: f64) {
        self.volume_button.set_value(volume);
    }

    fn set_song_title(&self, title: &str) {
        if title != "" {
            self.subtitle_label.set_text(title);
            self.subtitle_revealer.set_reveal_child(true);
        } else {
            self.subtitle_label.set_text("");
            self.subtitle_revealer.set_reveal_child(false);
        }
    }
}
