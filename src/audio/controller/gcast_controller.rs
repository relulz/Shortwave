use rust_cast::channels::media::{Media, StreamType};
use rust_cast::channels::receiver::CastDeviceApp;
use rust_cast::CastDevice;
use rust_cast::channels::media::GenericMediaMetadata;
use std::str::FromStr;
use rust_cast::channels::media::Image;
use rust_cast::channels::receiver::Application;

use std::rc::Rc;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use crate::api::Station;
use crate::app::Action;
use crate::audio::{GCastDevice, Controller, PlaybackState};

enum GCastAction{
    Connect,
    SetStation,
    SetPlaybackState,
    Disconnect,
}

pub struct GCastController{
    station: Arc<Mutex<Option<Station>>>,
    device_ip: Arc<Mutex<String>>,
    playback_state: Arc<Mutex<PlaybackState>>,

    gcast_sender: Sender<GCastAction>,
    app_sender: glib::Sender<Action>,
}

impl GCastController {
    pub fn new(app_sender: glib::Sender<Action>) -> Rc<Self> {
        let station = Arc::new(Mutex::new(None));
        let device_ip = Arc::new(Mutex::new("".to_string()));
        let playback_state = Arc::new(Mutex::new(PlaybackState::Stopped));

        let (gcast_sender, gcast_receiver) = channel();
        let gcast_controller = Self {
            station,
            device_ip,
            playback_state,

            gcast_sender,
            app_sender,
        };

        let gcc = Rc::new(gcast_controller);
        gcc.start_thread(gcast_receiver);
        gcc
    }

    fn start_thread(&self, receiver: Receiver<GCastAction>){
        let station = self.station.clone();
        let playback_state = self.playback_state.clone();
        let device_ip = self.device_ip.clone();

        let gcast_sender = self.gcast_sender.clone();

        thread::spawn(move || {
            let mut device: Option<CastDevice> = None;
            let mut app: Option<Application> = None;
            let mut media_session_id: i32 = 0;

            loop {
                if let Ok(action) = receiver.try_recv() {
                    match action{
                        GCastAction::Connect => {
                            debug!("Connect to gcast device with IP \"{}\"...", *device_ip.lock().unwrap());
                            match CastDevice::connect_without_host_verification(device_ip.lock().unwrap().to_string(), 8009){
                                Ok(d) => {
                                    d.connection.connect("receiver-0".to_string()).unwrap();
                                    d.heartbeat.ping().unwrap();

                                    let app_to_launch = CastDeviceApp::from_str("default").unwrap();
                                    let a = d.receiver.launch_app(&app_to_launch).unwrap();
                                    d.connection.connect(a.transport_id.as_str()).unwrap();

                                    debug!("Connected to gcast device!");
                                    device = Some(d);
                                    app = Some(a);

                                    // We need to set the station, since it already got set before.
                                    gcast_sender.send(GCastAction::SetStation).unwrap();
                                },
                                Err(_) => {
                                    warn!("Could not connect to gcast device.")
                                }

                            }
                        },
                        GCastAction::SetStation => {
                            device.as_ref().map(|d| {
                                let s = station.lock().unwrap().as_ref().unwrap().clone();

                                let image = Image{
                                    url: s.favicon.unwrap().to_string(),
                                    dimensions: None,
                                };

                                let metadata = GenericMediaMetadata{
                                    title: Some(s.name),
                                    subtitle: None,
                                    images: vec![image],
                                    release_date: None,
                                };

                                debug!("Transfer media information to gcast device...");
                                let status = d.media.load(
                                    app.as_ref().unwrap().transport_id.as_str(),
                                    app.as_ref().unwrap().session_id.as_str(),
                                    &Media {
                                        content_id: s.url.unwrap().to_string(),
                                        content_type: "".to_string(),
                                        stream_type: StreamType::Live,
                                        duration: None,
                                        metadata: Some(rust_cast::channels::media::Metadata::Generic(metadata)),
                                    },
                                ).unwrap();

                                let status = status.entries.first().unwrap();
                                media_session_id = status.media_session_id;
                            });
                        }
                        GCastAction::SetPlaybackState => {
                            device.as_ref().map(|d| {
                                let state = &*playback_state.lock().unwrap();

                                debug!("Update playback state of gcast device: {:?}", state);
                                let transport_id = app.as_ref().unwrap().transport_id.as_str();

                                match state {
                                    PlaybackState::Playing => { d.media.play(transport_id, media_session_id).expect("Could not set gcast state"); },
                                    PlaybackState::Stopped => { d.media.pause(transport_id, media_session_id).expect("Could not set gcast state"); },
                                    _ => (),
                                }
                            });
                        },
                        GCastAction::Disconnect => {
                            device.as_ref().map(|d| {
                                debug!("Disconnect from gcast device...");
                                d.receiver.stop_app(app.as_ref().unwrap().session_id.as_str()).unwrap();
                            });
                        },
                    }
                }
            }
        });
    }

    pub fn connect_to_device(&self, device: GCastDevice){
        *self.device_ip.lock().unwrap() = device.ip.to_string();
        self.gcast_sender.send(GCastAction::Connect).unwrap();
    }
}

impl Controller for Rc<GCastController> {
    fn set_station(&self, station: Station) {
        self.gcast_sender.send(GCastAction::Disconnect).unwrap();
        *self.station.lock().unwrap() = Some(station);
        self.gcast_sender.send(GCastAction::SetStation).unwrap();
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        // Check if state really changed
        if playback_state != &*self.playback_state.lock().unwrap() {
            *self.playback_state.lock().unwrap() = playback_state.clone();
            self.gcast_sender.send(GCastAction::SetPlaybackState).unwrap();
        }
    }

    fn set_volume(&self, _volume: f64) {

    }

    fn set_song_title(&self, _title: &str) {

    }
}
