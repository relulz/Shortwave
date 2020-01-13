use futures_util::future::FutureExt;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use url::Url;

use std::cell::RefCell;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::api::{Client, Station};
use crate::app::Action;
use crate::audio::backend::*;
use crate::audio::controller::{Controller, GCastController, MiniController, MprisController, SidebarController};
use crate::audio::{GCastDevice, Song};
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
    Loading,
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

        player.setup_signals(gst_receiver);
        player
    }

    pub fn set_station(&self, station: Station) {
        self.set_playback(PlaybackState::Stopped);

        // Station is broken, we refuse to play it
        if !station.lastcheckok {
            let notification = Notification::new_info("This station cannot be played because the stream is offline.");
            self.sender.send(Action::ViewShowNotification(notification)).unwrap();
            return;
        }

        for con in &*self.controller {
            con.set_station(station.clone());
        }

        let gst_backend = self.gst_backend.clone();
        let sender = self.sender.clone();
        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());
        // get asynchronously the stream url and play it
        let fut = client.get_stream_url(station).map(move |station_url| match station_url {
            Ok(station_url) => {
                debug!("new source uri to record: {}", station_url.url);
                gst_backend.lock().unwrap().new_source_uri(&station_url.url);
            }
            Err(err) => {
                let notification = Notification::new_error("Could not play station", &err.to_string());
                sender.send(Action::ViewShowNotification(notification)).unwrap();
            }
        });

        let ctx = glib::MainContext::default();
        ctx.spawn_local(fut);
    }

    pub fn set_playback(&self, playback: PlaybackState) {
        match playback {
            PlaybackState::Playing => {
                let _ = self.gst_backend.lock().unwrap().set_state(gstreamer::State::Playing);
            }
            PlaybackState::Stopped => {
                let _ = self.gst_backend.lock().unwrap().set_state(gstreamer::State::Null);
            }
            _ => (),
        }
    }

    pub fn set_volume(&self, volume: f64) {
        self.gst_backend.lock().unwrap().set_volume(volume.clone());

        for con in &*self.controller {
            con.set_volume(volume);
        }
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
        let sender = self.sender.clone();
        get_widget!(self.builder, gtk::Button, disconnect_button);
        disconnect_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackDisconnectGCastDevice).unwrap();
        });
    }

    fn process_gst_message(message: GstreamerMessage, controller: Rc<Vec<Box<dyn Controller>>>, song_backend: Rc<RefCell<SongBackend>>, gst_backend: Arc<Mutex<GstreamerBackend>>) -> glib::Continue {
        match message {
            GstreamerMessage::SongTitleChanged(title) => {
                debug!("Song title has changed: \"{}\"", title);

                for con in &*controller {
                    con.set_song_title(&title);
                }

                // Song have changed -> stop recording
                if gst_backend.lock().unwrap().is_recording() {
                    let song = gst_backend.lock().unwrap().stop_recording(true).unwrap();
                    song_backend.borrow_mut().add_song(song);
                } else {
                    // Get current/new song title
                    let title = gst_backend.lock().unwrap().get_current_song_title();

                    // Nothing needs to be stopped, so we can start directly recording.
                    gst_backend.lock().unwrap().start_recording(Self::get_song_path(title));
                }
            }
            GstreamerMessage::RecordingStopped => {
                // Recording successfully stopped.
                debug!("Recording stopped.");

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
