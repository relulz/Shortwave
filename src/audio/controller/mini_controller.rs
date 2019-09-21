use glib::Sender;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::Station;
use crate::app::Action;
use crate::audio::Controller;
use crate::audio::PlaybackState;

pub struct MiniController {
    pub widget: gtk::Box,
    sender: Sender<Action>,
    station: Rc<RefCell<Option<Station>>>,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    action_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    show_player_button: gtk::Button,
}

impl MiniController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/mini_controller.ui");

        let station = Rc::new(RefCell::new(None));

        let widget: gtk::Box = get_widget!(builder, "mini_controller");
        let title_label: gtk::Label = get_widget!(builder, "title_label");
        let subtitle_label: gtk::Label = get_widget!(builder, "subtitle_label");
        let subtitle_revealer: gtk::Revealer = get_widget!(builder, "subtitle_revealer");
        let action_revealer: gtk::Revealer = get_widget!(builder, "action_revealer");
        let playback_button_stack: gtk::Stack = get_widget!(builder, "playback_button_stack");
        let start_playback_button: gtk::Button = get_widget!(builder, "start_playback_button");
        let stop_playback_button: gtk::Button = get_widget!(builder, "stop_playback_button");
        let show_player_button: gtk::Button = get_widget!(builder, "show_player_button");

        let controller = Self {
            widget,
            sender,
            station,
            title_label,
            subtitle_label,
            action_revealer,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            show_player_button,
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
        self.show_player_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowPlayer).unwrap();
        });
    }
}

impl Controller for MiniController {
    fn set_station(&self, station: Station) {
        self.action_revealer.set_reveal_child(true);
        self.title_label.set_text(&station.name);
        self.title_label.set_tooltip_text(Some(station.name.as_str()));
        *self.station.borrow_mut() = Some(station);

        // reset everything else
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
