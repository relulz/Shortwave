use glib::Sender;
use glib::futures::FutureExt;
use gtk::prelude::*;
use gio::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::api::{Station, FaviconDownloader};
use crate::app::Action;
use crate::audio::Controller;
use crate::audio::PlaybackState;
use crate::ui::{StationDialog, StationFavicon, FaviconSize};

pub struct SidebarController {
    pub widget: gtk::Box,
    sender: Sender<Action>,
    station: Rc<RefCell<Option<Station>>>,
    app: gtk::Application,

    station_favicon: Rc<StationFavicon>,
    favicon_downloader: FaviconDownloader,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    subtitle_revealer: gtk::Revealer,
    action_revealer: gtk::Revealer,
    playback_button_stack: gtk::Stack,
    start_playback_button: gtk::Button,
    stop_playback_button: gtk::Button,
    error_label: gtk::Label,
    volume_button: gtk::VolumeButton,
    volume_signal_id: glib::signal::SignalHandlerId,

    action_group: gio::SimpleActionGroup,
}

impl SidebarController {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/sidebar_controller.ui");
        get_widget!(builder, gtk::Box, sidebar_controller);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, subtitle_label);
        get_widget!(builder, gtk::Revealer, subtitle_revealer);
        get_widget!(builder, gtk::Revealer, action_revealer);
        get_widget!(builder, gtk::Stack, playback_button_stack);
        get_widget!(builder, gtk::Button, start_playback_button);
        get_widget!(builder, gtk::Button, stop_playback_button);
        get_widget!(builder, gtk::Label, error_label);
        get_widget!(builder, gtk::VolumeButton, volume_button);

        let station = Rc::new(RefCell::new(None));
        let app = builder.get_application().unwrap();

        get_widget!(builder, gtk::Box, favicon_box);
        let station_favicon = Rc::new(StationFavicon::new(FaviconSize::Big));
        favicon_box.add(&station_favicon.widget);
        let favicon_downloader = FaviconDownloader::new();

        // volume_button | We need the volume_signal_id later to block the signal
        let s = sender.clone();
        let volume_signal_id = volume_button.connect_value_changed(move|_, value| {
            s.send(Action::PlaybackSetVolume(value)).unwrap();
        });

        // menu button
        let menu_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/menu/player_menu.ui");
        get_widget!(menu_builder, gtk::PopoverMenu, popover_menu);
        get_widget!(builder, gtk::MenuButton, playermenu_button);
        playermenu_button.set_popover(Some(&popover_menu));

        // action group
        let action_group = gio::SimpleActionGroup::new();
        sidebar_controller.insert_action_group("player", Some(&action_group));

        let controller = Self {
            widget: sidebar_controller,
            sender,
            station,
            app,
            station_favicon,
            favicon_downloader,
            title_label,
            subtitle_label,
            action_revealer,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            error_label,
            volume_button,
            volume_signal_id,
            action_group,
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

        // details button
        let station = self.station.clone();
        let app = self.app.clone();
        let sender = self.sender.clone();
        let details_action = gio::SimpleAction::new("show-details", None);
        details_action.connect_activate(move |_,_|{
            let window = app.get_active_window().unwrap();
            let s = station.borrow().clone().unwrap();

            let station_dialog = StationDialog::new(sender.clone(), s, &window);
            station_dialog.show();
        });
        self.action_group.add_action(&details_action);

        // stream button
        let sender = self.sender.clone();
        let stream_action = gio::SimpleAction::new("stream-audio", None);
        stream_action.connect_activate(move |_,_|{
            sender.send(Action::ViewShowStreamDialog).unwrap();
        });
        self.action_group.add_action(&stream_action);
    }
}

impl Controller for SidebarController {
    fn set_station(&self, station: Station) {
        self.action_revealer.set_reveal_child(true);
        self.title_label.set_text(&station.name);
        self.title_label.set_tooltip_text(Some(station.name.as_str()));
        *self.station.borrow_mut() = Some(station.clone());

        // Download & set icon
        let station_favicon = self.station_favicon.clone();
        station.favicon.map(|favicon| {
            let fut = self.favicon_downloader.clone().download (favicon, FaviconSize::Big as i32).map(move|pixbuf|{
                pixbuf.ok().map(|pixbuf| station_favicon.set_pixbuf(pixbuf));
            });
            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);
        });

        // reset everything else
        self.error_label.set_text(" ");
        self.station_favicon.reset();
        self.subtitle_revealer.set_reveal_child(false);
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        match playback_state {
            PlaybackState::Playing => self.playback_button_stack.set_visible_child_name("stop_playback"),
            PlaybackState::Stopped => self.playback_button_stack.set_visible_child_name("start_playback"),
            PlaybackState::Loading => self.playback_button_stack.set_visible_child_name("loading"),
            PlaybackState::Failure(msg) => {
                self.playback_button_stack.set_visible_child_name("error");
                let mut text = self.error_label.get_text().unwrap().to_string();
                text = text + " " + msg;
                self.error_label.set_text(&text);
            }
        };
    }

    fn set_volume(&self, volume: f64) {
        // We need to block the signal, otherwise we risk creating a endless loop
        glib::signal::signal_handler_block(&self.volume_button, &self.volume_signal_id);
        self.volume_button.set_value(volume);
        glib::signal::signal_handler_unblock(&self.volume_button, &self.volume_signal_id);
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
