// Shortwave - player.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use glib::{Receiver, Sender};
use gtk::prelude::*;

use std::cell::RefCell;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::api::Station;
use crate::app::Action;
use crate::audio::backend::*;
use crate::audio::controller::{Controller, GCastController, MiniController, MprisController, SidebarController};
use crate::audio::{GCastDevice, Song};
use crate::i18n::*;
use crate::path;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;
use crate::utils;

/////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                             //
//  A small overview of the player/gstreamer program structure  :)                             //
//                                                                                             //
//   -----------------    -----------------    -------------------    ----------------         //
//  | GCastController |  | MprisController |  | SidebarController |  | MiniController |        //
//   -----------------    -----------------    -------------------    ----------------         //
//            |                        |                   |                      |            //
//            \-------------------------------------------------------------------/            //
//                                     |                                                       //
//                           ------------     -------------------     --------------           //
//                          | Controller |   | Gstreamer Backend |   | Song Backend |          //
//                           ------------     -------------------     --------------           //
//                                     \______ | _______________________/                      //
//                                            \|/                                              //
//                                        -----------                                          //
//                                       |  Player   |                                         //
//                                        -----------                                          //
//                                                                                             //
/////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug)]
pub enum PlaybackState {
    Playing,
    Stopped,
    Failure(String),
}

pub struct Player {
    pub widget: gtk::Box,
    pub mini_controller_widget: gtk::Box,
    controller: Rc<Vec<Box<dyn Controller>>>,
    gcast_controller: Rc<GCastController>,

    gst_backend: Arc<Mutex<GstreamerBackend>>,
    song_backend: Rc<RefCell<SongBackend>>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Player {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/player.ui");
        get_widget!(builder, gtk::Box, player);
        let mut controller: Vec<Box<dyn Controller>> = Vec::new();

        // Sidebar Controller
        let sidebar_controller = SidebarController::new(sender.clone());
        get_widget!(builder, gtk::Box, player_box);
        player_box.add(&sidebar_controller.widget);
        player_box.reorder_child(&sidebar_controller.widget, 0);
        controller.push(Box::new(sidebar_controller));

        // Mini Controller (gets used in phone mode / bottom toolbar)
        let mini_controller = MiniController::new(sender.clone());
        let mini_controller_widget = mini_controller.widget.clone();
        controller.push(Box::new(mini_controller));

        // Mpris Controller
        let mpris_controller = MprisController::new(sender.clone());
        controller.push(Box::new(mpris_controller));

        // Google Cast Controller
        let gcast_controller = GCastController::new();
        controller.push(Box::new(gcast_controller.clone()));

        // Song backend + Widget
        let save_count: usize = settings_manager::get_integer(Key::RecorderSaveCount).try_into().unwrap();
        let song_backend = Rc::new(RefCell::new(SongBackend::new(sender.clone(), save_count)));
        song_backend.borrow().delete_songs(); // Delete old songs
        player_box.add(&song_backend.borrow().listbox.widget);
        player_box.reorder_child(&song_backend.borrow().listbox.widget, 3);

        // Gstreamer backend
        let (gst_sender, gst_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let gst_backend = Arc::new(Mutex::new(GstreamerBackend::new(gst_sender, sender.clone())));

        let controller: Rc<Vec<Box<dyn Controller>>> = Rc::new(controller);

        let player = Self {
            widget: player,
            mini_controller_widget,
            controller,
            gcast_controller,
            gst_backend,
            song_backend,
            builder,
            sender,
        };

        // Set volume
        let volume = settings_manager::get_double(Key::PlaybackVolume);
        player.set_volume(volume);

        player.setup_signals(gst_receiver);
        player
    }

    pub fn set_station(&self, station: Station) {
        self.set_playback(PlaybackState::Stopped);

        // Station is broken, we refuse to play it
        if station.lastcheckok != 1 {
            let notification = Notification::new_info(&i18n("This station cannot be played because the stream is offline."));
            send!(self.sender, Action::ViewShowNotification(notification));
            return;
        }

        for con in &*self.controller {
            con.set_station(station.clone());
        }

        match station.url_resolved {
            Some(url) => {
                debug!("Start playing new URI: {}", url.to_string());
                self.gst_backend.lock().unwrap().new_source_uri(&url.to_string());
            }
            None => {
                let notification = Notification::new_error(&i18n("Station cannot be streamed."), &i18n("URL is not valid."));
                send!(self.sender, Action::ViewShowNotification(notification));
            }
        }
    }

    pub fn set_playback(&self, playback: PlaybackState) {
        match playback {
            PlaybackState::Playing => {
                self.gst_backend.lock().unwrap().set_state(gstreamer::State::Playing);
            }
            PlaybackState::Stopped => {
                self.gst_backend.lock().unwrap().set_state(gstreamer::State::Null);
            }
            _ => (),
        }
    }

    pub fn set_volume(&self, volume: f64) {
        debug!("Set volume: {}", &volume);
        self.gst_backend.lock().unwrap().set_volume(volume.clone());

        for con in &*self.controller {
            con.set_volume(volume);
        }

        settings_manager::set_double(Key::PlaybackVolume, volume);
    }

    pub fn save_song(&self, song: Song) {
        self.song_backend.borrow().save_song(song);
    }

    pub fn connect_to_gcast_device(&self, device: GCastDevice) {
        get_widget!(self.builder, gtk::Label, device_name);
        get_widget!(self.builder, gtk::Revealer, stream_revealer);
        device_name.set_text(&format!("\"{}\"", &device.name));
        stream_revealer.set_reveal_child(true);

        self.gcast_controller.connect_to_device(device);
    }

    pub fn disconnect_from_gcast_device(&self) {
        get_widget!(self.builder, gtk::Revealer, stream_revealer);
        stream_revealer.set_reveal_child(false);
        self.gcast_controller.disconnect_from_device();
    }

    fn setup_signals(&self, receiver: Receiver<GstreamerMessage>) {
        // Wait for new messages from the Gstreamer backend
        let controller = self.controller.clone();
        let song_backend = self.song_backend.clone();
        let gst_backend = self.gst_backend.clone();
        receiver.attach(None, move |message| Self::process_gst_message(message, controller.clone(), song_backend.clone(), gst_backend.clone()));

        // Disconnect from gcast device
        get_widget!(self.builder, gtk::Button, disconnect_button);
        disconnect_button.connect_clicked(clone!(@strong self.sender as sender => move |_| {
            send!(sender, Action::PlaybackDisconnectGCastDevice);
        }));
    }

    fn process_gst_message(message: GstreamerMessage, controller: Rc<Vec<Box<dyn Controller>>>, song_backend: Rc<RefCell<SongBackend>>, gst_backend: Arc<Mutex<GstreamerBackend>>) -> glib::Continue {
        match message {
            GstreamerMessage::SongTitleChanged(title) => {
                debug!("Song title has changed to: \"{}\"", title);

                for con in &*controller {
                    con.set_song_title(&title);
                }

                if gst_backend.lock().unwrap().is_recording() {
                    // If we're already recording something, we need to stop it first.
                    // We cannot start recording the new song immediately.

                    if let Some(song) = gst_backend.lock().unwrap().stop_recording(true) {
                        // Add the recorded song to the song backend,
                        // which is responsible for the file management.
                        song_backend.borrow_mut().add_song(song);
                    }
                } else {
                    // If we don't record anything, we can start recording the new song
                    // immediately.

                    // Get current/new song title
                    let title = gst_backend.lock().unwrap().get_current_song_title();

                    // Nothing needs to be stopped, so we can start directly recording.
                    gst_backend.lock().unwrap().start_recording(Self::get_song_path(title));
                }
            }
            GstreamerMessage::RecordingStopped => {
                // We got the confirmation that the old recording stopped,
                // so we can start recording the new one now.
                debug!("Recording is stopped.");

                // Get current/new song title
                let title = gst_backend.lock().unwrap().get_current_song_title();

                // Start recording new song
                if title != "" {
                    gst_backend.lock().unwrap().start_recording(Self::get_song_path(title));
                }
            }
            GstreamerMessage::PlaybackStateChanged(state) => {
                for con in &*controller {
                    con.set_playback_state(&state);
                }

                if matches!(state, PlaybackState::Failure(_)) || matches!(state, PlaybackState::Stopped) {
                    // Discard current recording because the song has not yet been completely recorded.
                    gst_backend.lock().unwrap().stop_recording(false);
                }
            }
        }
        glib::Continue(true)
    }

    fn get_song_path(title: String) -> PathBuf {
        let title = utils::simplify_string(title);

        let mut path = path::CACHE.clone();
        path.push("recording");

        // Make sure that the path exists
        fs::create_dir_all(path.clone()).expect("Could not create path for recording");

        if title != "" {
            path.push(title);
            path.set_extension("ogg");
        }
        path.to_path_buf()
    }
}
