use rust_cast::channels::media::GenericMediaMetadata;
use rust_cast::channels::media::Image;
use rust_cast::channels::media::{Media, StreamType};
use rust_cast::channels::receiver::Application;
use rust_cast::channels::receiver::CastDeviceApp;
use rust_cast::CastDevice;
use std::str::FromStr;

use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::api::Station;
use crate::app::Action;
use crate::audio::{Controller, GCastDevice, PlaybackState};

enum GCastAction {
    Connect,
    SetStation,
    Disconnect,
}

pub struct GCastController {
    station: Arc<Mutex<Option<Station>>>,
    device_ip: Arc<Mutex<String>>,

    gcast_sender: Sender<GCastAction>,
    app_sender: glib::Sender<Action>,
}

impl GCastController {
    pub fn new(app_sender: glib::Sender<Action>) -> Rc<Self> {
        let station = Arc::new(Mutex::new(None));
        let device_ip = Arc::new(Mutex::new("".to_string()));

        let (gcast_sender, gcast_receiver) = channel();
        let gcast_controller = Self {
            station,
            device_ip,

            gcast_sender,
            app_sender,
        };

        let gcc = Rc::new(gcast_controller);
        gcc.start_thread(gcast_receiver);
        gcc
    }

    fn start_thread(&self, receiver: Receiver<GCastAction>) {
        let station = self.station.clone();
        let device_ip = self.device_ip.clone();

        let gcast_sender = self.gcast_sender.clone();

        thread::spawn(move || {
            let mut device: Option<CastDevice> = None;
            let mut app: Option<Application> = None;
            let mut media_session_id: i32 = 0;

            loop {
                if let Ok(action) = receiver.recv() {
                    match action {
                        GCastAction::Connect => {
                            debug!("Connect to gcast device with IP \"{}\"...", *device_ip.lock().unwrap());
                            match CastDevice::connect_without_host_verification(device_ip.lock().unwrap().to_string(), 8009) {
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
                                }
                                Err(_) => warn!("Could not connect to gcast device."),
                            }
                        }
                        GCastAction::SetStation => {
                            device.as_ref().map(|d| {
                                let s = station.lock().unwrap().as_ref().unwrap().clone();
                                debug!("Transfer media information to gcast device...");

                                let image = Image {
                                    url: s.clone().favicon.unwrap().to_string(),
                                    dimensions: None,
                                };

                                let metadata = GenericMediaMetadata {
                                    title: Some(s.name.clone()),
                                    subtitle: None,
                                    images: vec![image],
                                    release_date: None,
                                };

                                let result = d.media.load(
                                    app.as_ref().unwrap().transport_id.as_str(),
                                    app.as_ref().unwrap().session_id.as_str(),
                                    &Media {
                                        content_id: s.url.clone().unwrap().to_string(),
                                        content_type: "".to_string(),
                                        stream_type: StreamType::Live,
                                        duration: None,
                                        metadata: Some(rust_cast::channels::media::Metadata::Generic(metadata)),
                                    },
                                );

                                match result {
                                    Ok(status) => {
                                        let status = status.entries.first().unwrap();
                                        media_session_id = status.media_session_id;
                                    }
                                    _ => warn!("Unable to transfer media information to gcast device."),
                                }
                            });
                        }
                        GCastAction::Disconnect => {
                            device.as_ref().map(|d| {
                                debug!("Disconnect from gcast device...");

                                match d.receiver.stop_app(app.as_ref().unwrap().session_id.as_str()) {
                                    Ok(_) => (),
                                    _ => warn!("Unable to disconnect from gcast device."),
                                }
                            });
                        }
                    }
                }
            }
        });
    }

    pub fn connect_to_device(&self, device: GCastDevice) {
        *self.device_ip.lock().unwrap() = device.ip.to_string();
        self.gcast_sender.send(GCastAction::Connect).unwrap();
    }

    pub fn disconnect_from_device(&self) {
        self.gcast_sender.send(GCastAction::Disconnect).unwrap();
    }
}

impl Controller for Rc<GCastController> {
    fn set_station(&self, station: Station) {
        self.gcast_sender.send(GCastAction::Disconnect).unwrap();
        *self.station.lock().unwrap() = Some(station);
        self.gcast_sender.send(GCastAction::SetStation).unwrap();
    }

    fn set_playback_state(&self, _playback_state: &PlaybackState) {
        // Ignore
    }

    fn set_volume(&self, _volume: f64) {
        // Ignore
    }

    fn set_song_title(&self, _title: &str) {
        // Ignore
    }
}