use gstreamer::prelude::*;
use gtk::prelude::*;
use mpris_player::{Metadata, MprisPlayer, OrgMprisMediaPlayer2Player, PlaybackStatus};
use rustio::{Client, Station};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::Action;
use crate::gstreamer_backend::PlayerBackend;
use crate::song::Song;
use crate::widgets::song_row::SongRow;

pub enum PlaybackState {
    Playing,
    Stopped,
    Loading,
}

struct PlayerWidgets {
    pub title_label: gtk::Label,
    pub subtitle_label: gtk::Label,
    pub subtitle_revealer: gtk::Revealer,
    pub playback_button_stack: gtk::Stack,
    pub start_playback_button: gtk::Button,
    pub stop_playback_button: gtk::Button,
    pub volume_button: gtk::VolumeButton,
}

impl PlayerWidgets {
    pub fn new(builder: gtk::Builder) -> Self {
        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        let subtitle_label: gtk::Label = builder.get_object("subtitle_label").unwrap();
        let subtitle_revealer: gtk::Revealer = builder.get_object("subtitle_revealer").unwrap();
        let playback_button_stack: gtk::Stack = builder.get_object("playback_button_stack").unwrap();
        let start_playback_button: gtk::Button = builder.get_object("start_playback_button").unwrap();
        let stop_playback_button: gtk::Button = builder.get_object("stop_playback_button").unwrap();
        let volume_button: gtk::VolumeButton = builder.get_object("volume_button").unwrap();

        PlayerWidgets {
            title_label,
            subtitle_label,
            subtitle_revealer,
            playback_button_stack,
            start_playback_button,
            stop_playback_button,
            volume_button,
        }
    }

    pub fn reset(&self) {
        self.title_label.set_text("");
        self.subtitle_label.set_text("");
        self.subtitle_revealer.set_reveal_child(false);
    }

    pub fn set_title(&self, title: &str) {
        if title != "" {
            self.subtitle_label.set_text(title);
            self.subtitle_revealer.set_reveal_child(true);
        } else {
            self.subtitle_label.set_text("");
            self.subtitle_revealer.set_reveal_child(false);
        }
    }
}

pub struct SongHistory {
    pub current_station: Option<Station>,
    pub current_song: Option<Song>,
    pub history: Vec<Song>,
    pub max_history: usize,


    song_rows: Vec<SongRow>,
    last_played_listbox: gtk::ListBox, //TODO: rename
    recording_box: gtk::Box,
}

impl SongHistory {
    pub fn new(builder: gtk::Builder) -> Self {
        let current_station = None;
        let current_song = None;
        let history = Vec::new();
        let max_history = 10;

        let song_rows = Vec::new();
        let last_played_listbox: gtk::ListBox = builder.get_object("last_played_listbox").unwrap();
        let recording_box: gtk::Box = builder.get_object("recording_box").unwrap();

        Self {
            current_station,
            current_song,
            history,
            max_history,
            song_rows,
            last_played_listbox,
            recording_box,
        }
    }

    pub fn discard_current_song(&mut self) {
        self.current_song.take().map(|mut song| song.delete());
    }

    // returns 'true' if song have changed in comparsion to old song
    pub fn set_new_song(&mut self, song: Song) -> bool {
        // check if song have changed
        if self.current_song != Some(song.clone()) {
            // save current song, and insert it into the history
            self.current_song.take().map(|mut s| {
                s.finish();
                let row = SongRow::new(s.clone());

                self.last_played_listbox.insert(&row.widget, 0);
                self.song_rows.insert(0, row);
                self.history.insert(0, s);

                self.recording_box.set_visible(true);
            });

            // set new current_song
            self.current_song = Some(song);

            // ensure max history length. Delete old songs
            if self.history.len() > self.max_history{
                self.history.pop().map(|mut song|{
                    song.delete();
                    self.last_played_listbox.remove(&self.song_rows.pop().unwrap().widget);
                });
            }
            return true;
        }
        false
    }

    pub fn get_previous_song(&self) -> Option<&Song> {
        self.history.get(0)
    }

    pub fn delete_everything(&mut self){
        self.discard_current_song();

        for song in &mut self.history{
            song.delete();
        }
        self.history.clear();
    }
}

pub struct Player {
    pub widget: gtk::Box,
    player_widgets: Rc<PlayerWidgets>,

    backend: Arc<Mutex<PlayerBackend>>,
    mpris: Arc<MprisPlayer>,
    song_history: Rc<RefCell<SongHistory>>,

    sender: Sender<Action>,
}

impl Player {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/player.ui");
        let widget: gtk::Box = builder.get_object("player").unwrap();
        let player_widgets = Rc::new(PlayerWidgets::new(builder.clone()));
        let backend = Arc::new(Mutex::new(PlayerBackend::new()));
        let song_history = Rc::new(RefCell::new(SongHistory::new(builder.clone())));

        let mpris = MprisPlayer::new("Shortwave".to_string(), "Shortwave".to_string(), "de.haeckerfelix.Shortwave".to_string());
        mpris.set_can_raise(true);
        mpris.set_can_play(false);
        mpris.set_can_seek(false);
        mpris.set_can_set_fullscreen(false);
        mpris.set_can_pause(true);

        let player = Self {
            widget,
            player_widgets,
            backend,
            mpris,
            song_history,
            sender,
        };

        player.setup_signals();
        player
    }

    pub fn set_station(&self, station: Station) {
        // discard old song, because it's not completely recorded
        self.song_history.borrow_mut().discard_current_song();

        self.player_widgets.reset();
        self.player_widgets.title_label.set_text(&station.name);
        self.song_history.borrow_mut().current_station = Some(station.clone());
        self.set_playback(PlaybackState::Stopped);

        // set mpris metadata
        let mut metadata = Metadata::new();
        metadata.art_url = Some(station.clone().favicon);
        metadata.artist = Some(vec![station.clone().name]);
        self.mpris.set_metadata(metadata);
        self.mpris.set_can_play(true);

        let backend = self.backend.clone();
        thread::spawn(move || {
            let mut client = Client::new("http://www.radio-browser.info");
            let station_url = client.get_playable_station_url(station).unwrap();
            debug!("new source uri to record: {}", station_url);
            backend.lock().unwrap().new_source_uri(&station_url);
        });
    }

    pub fn set_playback(&self, playback: PlaybackState) {
        match playback {
            PlaybackState::Playing => {
                let _ = self.backend.lock().unwrap().set_state(gstreamer::State::Playing);
            }
            PlaybackState::Stopped => {
                let _ = self.backend.lock().unwrap().set_state(gstreamer::State::Null);

                // We need to set it manually, because we don't receive a gst message when the playback stops
                self.player_widgets.playback_button_stack.set_visible_child_name("start_playback");
                self.mpris.set_playback_status(PlaybackStatus::Stopped);
            }
            _ => (),
        };
    }

    pub fn set_volume(&self, volume: f64) {
        self.backend.lock().unwrap().set_volume(volume);
    }

    pub fn shutdown(&self){
        self.set_playback(PlaybackState::Stopped);
        self.song_history.borrow_mut().delete_everything();
    }

    fn parse_bus_message(message: &gstreamer::Message, player_widgets: Rc<PlayerWidgets>, mpris: Arc<MprisPlayer>, backend: Arc<Mutex<PlayerBackend>>, song_history: Rc<RefCell<SongHistory>>) {
        match message.view() {
            gstreamer::MessageView::Tag(tag) => {
                tag.get_tags().get::<gstreamer::tags::Title>().map(|t| {
                    let new_song = Song::new(t.get().unwrap());

                    // Check if song have changed
                    if song_history.borrow_mut().set_new_song(new_song.clone()) {
                        // set new song
                        debug!("New song: {:?}", new_song.clone().title);
                        player_widgets.set_title(&new_song.clone().title);

                        // TODO: this would override the artist/art_url field. Needs to be fixed at mpris_player
                        // let mut metadata = Metadata::new();
                        // metadata.title = Some(title.get().unwrap().to_string());
                        // mpris.set_metadata(metadata);

                        debug!("Block the dataflow ...");
                        backend.lock().unwrap().block_dataflow();
                    }
                });
            }
            gstreamer::MessageView::StateChanged(sc) => {
                debug!("playback state changed: {:?}", sc.get_current());
                let playback_state = match sc.get_current() {
                    gstreamer::State::Playing => PlaybackState::Playing,
                    gstreamer::State::Paused => PlaybackState::Loading,
                    gstreamer::State::Ready => PlaybackState::Loading,
                    _ => PlaybackState::Stopped,
                };

                match playback_state {
                    PlaybackState::Playing => {
                        player_widgets.playback_button_stack.set_visible_child_name("stop_playback");
                        mpris.set_playback_status(PlaybackStatus::Playing);
                    }
                    PlaybackState::Stopped => {
                        player_widgets.playback_button_stack.set_visible_child_name("start_playback");
                        mpris.set_playback_status(PlaybackStatus::Stopped);
                    }
                    PlaybackState::Loading => {
                        player_widgets.playback_button_stack.set_visible_child_name("loading");
                        mpris.set_playback_status(PlaybackStatus::Stopped);
                    }
                };
            }
            gstreamer::MessageView::Element(element) => {
                let structure = element.get_structure().unwrap();
                if structure.get_name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap();
                    if let gstreamer::MessageView::Eos(_) = &message.view() {
                        debug!("muxsinkbin got EOS...");

                        if song_history.borrow().current_song.is_some() {
                            // Old song got saved correctly (cause we got the EOS message),
                            // so we can start with the new song now
                            let song = song_history.borrow_mut().current_song.clone().unwrap();
                            debug!("Cache song \"{}\" under \"{}\"", song.title, song.path);
                            backend.lock().unwrap().new_filesink_location(&song.path);
                        } else {
                            // Or just redirect the stream to /dev/null
                            backend.lock().unwrap().new_filesink_location("/dev/null");
                        }
                    }
                }
            }
            _ => (),
        };
    }

    fn setup_signals(&self) {
        // start_playback_button
        let sender = self.sender.clone();
        self.player_widgets.start_playback_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackStart).unwrap();
        });

        // stop_playback_button
        let sender = self.sender.clone();
        self.player_widgets.stop_playback_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackStop).unwrap();
        });

        // mpris raise
        let sender = self.sender.clone();
        self.mpris.connect_raise(move || {
            sender.send(Action::ViewRaise).unwrap();
        });

        // mpris play / pause
        let sender = self.sender.clone();
        let mpris = self.mpris.clone();
        self.mpris.connect_play_pause(move || {
            match mpris.get_playback_status().unwrap().as_ref() {
                "Paused" => sender.send(Action::PlaybackStart).unwrap(),
                "Stopped" => sender.send(Action::PlaybackStart).unwrap(),
                _ => sender.send(Action::PlaybackStop).unwrap(),
            };
        });

        // volume button
        let sender = self.sender.clone();
        self.player_widgets.volume_button.connect_value_changed(move |_, value| {
            sender.send(Action::PlaybackSetVolume(value)).unwrap();
        });

        // new backend (pipeline) bus messages
        let bus = self.backend.lock().unwrap().get_pipeline_bus();
        let player_widgets = self.player_widgets.clone();
        let backend = self.backend.clone();
        let song_history = self.song_history.clone();
        let mpris = self.mpris.clone();
        gtk::timeout_add(250, move || {
            while bus.have_pending() {
                bus.pop().map(|message| {
                    //debug!("new message {:?}", message);
                    Self::parse_bus_message(&message, player_widgets.clone(), mpris.clone(), backend.clone(), song_history.clone());
                });
            }
            Continue(true)
        });
    }
}
