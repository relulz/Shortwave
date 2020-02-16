use glib::Sender;
use gstreamer::prelude::*;
use gstreamer::{Bin, Element, ElementFactory, GhostPad, Pad, PadProbeId, Pipeline, State};

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::app::Action;
use crate::audio::PlaybackState;
use crate::audio::Song;

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                //
//  # Gstreamer Pipeline                                                                          //
//                                            -----      --------       -------------             //
//                                           |     | -> | queue [1] -> | recorderbin |            //
//    --------------      --------------     |     |     --------       -------------             //
//   | uridecodebin | -> | audioconvert | -> | tee |                                              //
//    --------------      --------------     |     |     -------      -----------                 //
//                                           |     | -> | queue | -> | pulsesink |                //
//                                            -----      -------      -----------                 //
//                                                                                                //
//                                                                                                //
//                                                                                                //
//  We use the the file_srcpad[1] to block the dataflow, so we can change the recorderbin.        //
//  The dataflow gets blocked when the song changes.                                              //
//                                                                                                //
//                                                                                                //
//  But how does recording work in detail?                                                        //
//                                                                                                //
//  1) We start recording a new song, when...                                                     //
//     a) The song title changed, and there's no current recording running                        //
//        [ player.rs -> process_gst_message() -> GstreamerMessage::SongTitleChanged ]            //
//     b) The song title changed, and the old recording stopped                                   //
//        [ player.rs -> process_gst_message() -> GstreamerMessage::RecordingStopped ]            //
//                                                                                                //
//  2) Before we can start recording, we need to ensure that the old recording is stopped.        //
//     This is usually not the case, except it's the first song we record.                        //
//     The recording gets stopped by calling "stop_recording()"                                   //
//     [ player.rs -> process_gst_message() -> GstreamerMessage::SongTitleChanged ]               //
//                                                                                                //
//  3) First of all, we have to make sure the old recorderbin gets destroyed. So we have          //
//     to block the pipeline first at [1], by using a block probe.                                //
//                                                                                                //
//  4) After the pipeline is blocked, we push a EOS event into the recorderbin sinkpad.           //
//     We need the EOS event, otherwise we cannot remove the old recorderbin from the             //
//     running pipeline. Without the EOS event, we would have to stop the whole pipeline.         //
//     With it we can dynamically add/remove recorderbins from the pipeline.                      //
//                                                                                                //
//  5) We detect the EOS event by listening to the pipeline bus. We confirm this by sending       //
//     the "GstreamerMessage::RecordingStopped" message.                                          //
//     [ gstreamer_backend.rs -> parse_bus_message() -> gstreamer::MessageView::Element() ]       //
//                                                                                                //
//  6) After we get this message, we can start recording the new song, by creating a new          //
//     recorderbin with "start_recording()"                                                       //
//     [ player.rs -> process_gst_message() -> GstreamerMessage::RecordingStopped() ]             //
//                                                                                                //
//  7) The recorderbin gets created and appendend to the pipeline. Now the stream gets            //
//     forwarded into a new file again.                                                           //
//                                                                                                //
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum GstreamerMessage {
    SongTitleChanged(String),
    PlaybackStateChanged(PlaybackState),
    RecordingStopped,
}

#[allow(dead_code)]
pub struct GstreamerBackend {
    pipeline: Pipeline,

    uridecodebin: Element,
    audioconvert: Element,
    tee: Element,

    audio_queue: Element,
    pulsesink: Element, // TODO: Is it good to hardcode pulsesink here instead of autoaudiosink?

    file_queue: Element,
    recorderbin: Arc<Mutex<Option<RecorderBin>>>,
    file_srcpad: Pad,
    file_blockprobe_id: Option<PadProbeId>,

    current_title: Arc<Mutex<String>>,

    volume: Arc<Mutex<f64>>,
    volume_signal_id: glib::signal::SignalHandlerId,
    sender: Sender<GstreamerMessage>,
}

impl GstreamerBackend {
    pub fn new(gst_sender: Sender<GstreamerMessage>, app_sender: Sender<Action>) -> Self {
        // create gstreamer pipeline
        let pipeline = Pipeline::new(Some("recorder_pipeline"));

        // create pipeline elements
        let uridecodebin = ElementFactory::make("uridecodebin", Some("uridecodebin")).unwrap();
        let audioconvert = ElementFactory::make("audioconvert", Some("audioconvert")).unwrap();
        let tee = ElementFactory::make("tee", Some("tee")).unwrap();
        let audio_queue = ElementFactory::make("queue", Some("audio_queue")).unwrap();
        let pulsesink = ElementFactory::make("pulsesink", Some("pulsesink")).expect("Could not find PulseAudio (Cannot create gstreamer `pulsesink` element).");
        let file_queue = ElementFactory::make("queue", Some("file_queue")).unwrap();
        let file_srcpad = file_queue.get_static_pad("src").unwrap();

        // link pipeline elements
        pipeline.add_many(&[&uridecodebin, &audioconvert, &tee, &audio_queue, &pulsesink, &file_queue]).unwrap();
        Element::link_many(&[&audioconvert, &tee]).unwrap();
        let tee_tempmlate = tee.get_pad_template("src_%u").unwrap();

        // link tee -> queue
        let tee_file_srcpad = tee.request_pad(&tee_tempmlate, None, None).unwrap();
        let _ = tee_file_srcpad.link(&file_queue.get_static_pad("sink").unwrap());

        // link tee -> queue -> pulsesink
        let tee_audio_srcpad = tee.request_pad(&tee_tempmlate, None, None).unwrap();
        let _ = tee_audio_srcpad.link(&audio_queue.get_static_pad("sink").unwrap());
        let _ = audio_queue.link(&pulsesink);

        let recorderbin = Arc::new(Mutex::new(None));

        // dynamically link uridecodebin element with audioconvert element
        let convert = audioconvert.clone();
        uridecodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = convert.get_static_pad("sink").expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                return; // We are already linked. Ignoring.
            }

            let new_pad_caps = src_pad.get_current_caps().expect("Failed to get caps of new pad.");
            let new_pad_struct = new_pad_caps.get_structure(0).expect("Failed to get first structure of caps.");
            let new_pad_type = new_pad_struct.get_name();

            if new_pad_type.starts_with("audio/x-raw") {
                // check if new_pad is audio
                let _ = src_pad.link(&sink_pad);
                return;
            }
        });

        // Current song title. We need this variable to check if the title have changed.
        let current_title = Arc::new(Mutex::new(String::new()));

        // listen for new pipeline / bus messages
        let ct = current_title.clone();
        let bus = pipeline.get_bus().expect("Unable to get pipeline bus");
        let s = gst_sender.clone();
        gtk::timeout_add(250, move || {
            while bus.have_pending() {
                bus.pop().map(|message| {
                    //debug!("new message {:?}", message);
                    Self::parse_bus_message(&message, s.clone(), ct.clone());
                });
            }
            Continue(true)
        });

        // We have to update the volume if we get changes from pulseaudio (pulsesink).
        // The user is able to control the volume from g-c-c.
        let volume = Arc::new(Mutex::new(1.0));
        let (volume_sender, volume_receiver) = glib::MainContext::channel(glib::PRIORITY_LOW);

        // We need to do message passing (sender/receiver) here, because gstreamer messages are
        // coming from a other thread (and app::Action enum is not thread safe).
        let a_s = app_sender.clone();
        volume_receiver.attach(None, move |volume| {
            a_s.send(Action::PlaybackSetVolume(volume)).unwrap();
            glib::Continue(true)
        });

        // Update volume coming from pulseaudio / pulsesink
        let old_volume = volume.clone();
        let v_s = volume_sender.clone();
        let volume_signal_id = pulsesink.connect_notify(Some("volume"), move |element, _| {
            let new_volume: f64 = element.get_property("volume").unwrap().get().unwrap().unwrap();

            // We have to check if the values are the same. For some reason gstreamer sends us
            // slightly differents floats, so we round up here (only the the first two digits are
            // important for use here).
            let new_val = format!("{:.2}", new_volume);
            let old_val = format!("{:.2}", old_volume.lock().unwrap());

            if new_val != old_val {
                v_s.send(new_volume).unwrap();
                *old_volume.lock().unwrap() = new_volume;
            }
        });

        // It's possible to mute the audio (!= 0.0) from pulseaudio side, so we should handle
        // this too by setting the volume to 0.0
        let old_volume = volume.clone();
        let v_s = volume_sender.clone();
        pulsesink.connect_notify(Some("mute"), move |element, _| {
            let mute: bool = element.get_property("mute").unwrap().get().unwrap().unwrap();
            if mute && *old_volume.lock().unwrap() != 0.0 {
                v_s.send(0.0).unwrap();
                *old_volume.lock().unwrap() = 0.0;
            }
        });

        let pipeline = Self {
            pipeline,
            uridecodebin,
            audioconvert,
            tee,
            audio_queue,
            pulsesink,
            file_queue,
            recorderbin,
            file_srcpad,
            file_blockprobe_id: None,
            current_title,
            volume,
            volume_signal_id,
            sender: gst_sender,
        };

        pipeline
    }

    pub fn set_state(&mut self, state: gstreamer::State) {
        if state == gstreamer::State::Null {
            self.sender.send(GstreamerMessage::PlaybackStateChanged(PlaybackState::Stopped)).unwrap();
        }

        let _ = self.pipeline.set_state(state);
    }

    pub fn set_volume(&self, volume: f64) {
        // We need to block the signal, otherwise we risk creating a endless loop
        glib::signal::signal_handler_block(&self.pulsesink, &self.volume_signal_id);
        *self.volume.lock().unwrap() = volume;
        self.pulsesink.set_property("volume", &volume).unwrap();
        glib::signal::signal_handler_unblock(&self.pulsesink, &self.volume_signal_id);
    }

    pub fn new_source_uri(&mut self, source: &str) {
        debug!("Stop pipeline...");
        let _ = self.pipeline.set_state(State::Null);

        debug!("Set new source URI...");
        self.uridecodebin.set_property("uri", &source).unwrap();

        debug!("Start pipeline...");
        let _ = self.pipeline.set_state(State::Playing);
    }

    pub fn start_recording(&mut self, path: PathBuf) {
        debug!("Start recording to {:?}", path);

        // We need to set an offset, otherwise the length of the recorded song would be wrong.
        // Get current clock time and calculate offset
        let clock = self.pipeline.get_clock().expect("Could not get gstreamer pipeline clock");
        debug!("( Clock time: {} )", clock.get_time());
        let offset = -(clock.get_time().nseconds().unwrap() as i64);
        self.file_srcpad.set_offset(offset);

        if self.recorderbin.lock().unwrap().is_some() {
            debug!("Destroyed old recorderbin.");
            self.recorderbin.lock().unwrap().take().unwrap().destroy();
        } else {
            debug!("No recorderbin available - nothing to destroy.");
        }

        debug!("Create new recorderbin.");
        let recorderbin = RecorderBin::new(self.get_current_song_title(), path, self.pipeline.clone(), &self.file_srcpad);
        *self.recorderbin.lock().unwrap() = Some(recorderbin);

        // Remove block probe id, if available
        match self.file_blockprobe_id.take() {
            Some(id) => {
                self.file_srcpad.remove_probe(id);
                debug!("Removed block probe.");
            }
            None => debug!("No block probe to remove."),
        }
    }

    pub fn stop_recording(&mut self, save_song: bool) -> Option<Song> {
        debug!("Stop recording... (Save song: {})", save_song);

        // Check if recorderbin is available
        if self.recorderbin.lock().unwrap().is_some() {
            let rbin = self.recorderbin.clone();

            // Check if we want to save the recorded data
            // Sometimes we can discard it as is got interrupted / not completely recorded
            if save_song {
                let file_id = self
                    .file_srcpad
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, move |_, _| {
                        // Dataflow is blocked
                        debug!("Push EOS into recorderbin sinkpad...");
                        //TODO: Fix crash here...| thread '<unnamed>' panicked at 'called `Option::unwrap()` on a `None` value', src/libcore/option.rs:378:21
                        let sinkpad = rbin.lock().unwrap().clone().unwrap().gstbin.get_static_pad("sink").unwrap();
                        sinkpad.send_event(gstreamer::Event::new_eos().build());

                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();

                // We need the padprobe id later to remove the block probe
                self.file_blockprobe_id = Some(file_id);

                // Create song and return it
                let song = self.recorderbin.lock().unwrap().clone().unwrap().stop();

                // Check song duration
                // Few stations are using the song metadata field as newstracker,
                // which means the text changes every few seconds.
                // Because of this reason, we shouldn't record songs with a too low duration.
                if song.duration > std::time::Duration::from_secs(20) {
                    return Some(song);
                } else {
                    info!("Ignore song \"{}\". Duration is not long enough.", song.title);
                    return None;
                }
            } else {
                // Discard recorded data
                debug!("Discard recorded data.");
                let recorderbin = self.recorderbin.lock().unwrap().take().unwrap();
                fs::remove_file(&recorderbin.song_path).expect("Could not delete recorded data");
                recorderbin.destroy();
                return None;
            }
        } else {
            debug!("No recorderbin available - nothing to stop.");
            return None;
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recorderbin.lock().unwrap().is_some()
    }

    pub fn get_current_song_title(&self) -> String {
        self.current_title.lock().unwrap().clone()
    }

    fn parse_bus_message(message: &gstreamer::Message, sender: Sender<GstreamerMessage>, current_title: Arc<Mutex<String>>) {
        match message.view() {
            gstreamer::MessageView::Tag(tag) => {
                tag.get_tags().get::<gstreamer::tags::Title>().map(|t| {
                    let new_title = t.get().unwrap().to_string();

                    // only send message if song title really have changed.
                    if *current_title.lock().unwrap() != new_title {
                        *current_title.lock().unwrap() = new_title.clone();
                        sender.send(GstreamerMessage::SongTitleChanged(new_title)).unwrap();
                    }
                });
            }
            gstreamer::MessageView::StateChanged(sc) => {
                let playback_state = match sc.get_current() {
                    gstreamer::State::Playing => PlaybackState::Playing,
                    gstreamer::State::Paused => PlaybackState::Playing,
                    gstreamer::State::Ready => PlaybackState::Playing,
                    _ => PlaybackState::Stopped,
                };

                sender.send(GstreamerMessage::PlaybackStateChanged(playback_state)).unwrap();
            }
            gstreamer::MessageView::Element(element) => {
                let structure = element.get_structure().unwrap();
                if structure.get_name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap().unwrap();
                    if let gstreamer::MessageView::Eos(_) = &message.view() {
                        // recorderbin got EOS which means the current song got successfully saved.
                        debug!("Recorderbin received EOS event.");
                        sender.send(GstreamerMessage::RecordingStopped).unwrap();
                    }
                }
            }
            gstreamer::MessageView::Error(err) => {
                let msg = err.get_error().to_string();
                warn!("Gstreamer Error: {:?}", msg);
                sender.send(GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(msg))).unwrap();
            }
            _ => (),
        };
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                      //
//  # RecorderBin                                                                                       //
//                                                                                                      //
//    --------------------------------------------------------------                                    //
//   |                  -----------       --------      ----------  |                                   //
//   | ( ghostpad ) -> | vorbisenc | ->  | oggmux | -> | filesink | |                                   //
//   |                  -----------       --------      ----------  |                                   //
//    --------------------------------------------------------------                                    //
//                                                                                                      //
/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Clone)]
struct RecorderBin {
    pub gstbin: Bin,
    pipeline: Pipeline,

    ghostpad: GhostPad,
    vorbisenc: Element,
    oggmux: Element,
    filesink: Element,

    song_title: String,
    pub song_path: PathBuf,
    song_timestamp: SystemTime,
}

impl RecorderBin {
    pub fn new(song_title: String, song_path: PathBuf, pipeline: Pipeline, srcpad: &Pad) -> Self {
        // Create elements
        let vorbisenc = ElementFactory::make("vorbisenc", Some("vorbisenc")).unwrap();
        let oggmux = ElementFactory::make("oggmux", Some("oggmux")).unwrap();
        let filesink = ElementFactory::make("filesink", Some("filesink")).unwrap();
        filesink.set_property("location", &song_path.to_str().unwrap()).unwrap();

        // Create bin itself
        let bin = Bin::new(Some("bin"));
        bin.set_property("message-forward", &true).unwrap();

        // Add elements to bin and link them
        bin.add(&vorbisenc).unwrap();
        bin.add(&oggmux).unwrap();
        bin.add(&filesink).unwrap();
        Element::link_many(&[&vorbisenc, &oggmux, &filesink]).unwrap();

        // Add bin to pipeline
        pipeline.add(&bin).expect("Could not add recorderbin to pipeline");

        // Link file_srcpad with vorbisenc sinkpad using a ghostpad
        let vorbisenc_sinkpad = vorbisenc.get_static_pad("sink").unwrap();
        let ghostpad = gstreamer::GhostPad::new(Some("sink"), &vorbisenc_sinkpad).unwrap();
        bin.add_pad(&ghostpad).unwrap();
        bin.sync_state_with_parent().unwrap();
        srcpad.link(&ghostpad).expect("Queue src pad cannot linked to vorbisenc sinkpad");

        // Set song timestamp so we can check the duration later
        let song_timestamp = SystemTime::now();

        Self {
            gstbin: bin,
            pipeline,
            ghostpad,
            vorbisenc,
            oggmux,
            filesink,
            song_title,
            song_path,
            song_timestamp,
        }
    }

    pub fn stop(&self) -> Song {
        let now = SystemTime::now();
        let duration = now.duration_since(self.song_timestamp).unwrap();

        Song::new(&self.song_title, self.song_path.clone(), duration)
    }

    pub fn destroy(&self) {
        self.pipeline.remove(&self.gstbin).unwrap();
        self.gstbin.set_state(State::Null).unwrap();
    }
}
