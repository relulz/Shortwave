// Shortwave - gstreamer_backend.rs
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

use glib::Sender;
use gstreamer::prelude::*;
use gstreamer::{Bin, Element, Event, MessageView, PadProbeReturn, PadProbeType, Pipeline, State};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app::Action;
use crate::audio::PlaybackState;

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                //
//  # Gstreamer Pipeline                                                                          //
//                                            -----     (   -------------   )                     //
//                                           |     | -> (  | recorderbin |  )                     //
//    --------------      --------------     |     |    (   -------------   )                     //
//   | uridecodebin | -> | audioconvert | -> | tee |                                              //
//    --------------      --------------     |     |     -------      -----------                 //
//                                           |     | -> | queue | -> | pulsesink |                //
//                                            -----      -------      -----------                 //
//                                                                                                //
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum GstreamerMessage {
    SongTitleChanged(String),
    PlaybackStateChanged(PlaybackState),
}

pub struct GstreamerBackend {
    pipeline: Pipeline,
    recorderbin: Arc<Mutex<Option<Bin>>>,
    current_title: Arc<Mutex<String>>,
    volume: Arc<Mutex<f64>>,
    volume_signal_id: Option<glib::signal::SignalHandlerId>,
    sender: Sender<GstreamerMessage>,
}

impl GstreamerBackend {
    pub fn new(gst_sender: Sender<GstreamerMessage>, app_sender: Sender<Action>) -> Self {
        // create gstreamer pipeline

        let pipeline =
            gstreamer::parse_launch("uridecodebin name=uridecodebin ! audioconvert name=audioconvert ! tee name=tee ! queue ! pulsesink name=pulsesink").expect("Could not create gstreamer pipeline");
        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().expect("Couldn't downcast pipeline");
        pipeline.set_property_message_forward(true);

        // The recorderbin gets added / removed dynamically to the pipeline
        let recorderbin = Arc::new(Mutex::new(None));

        // Current song title
        // We need this variable to check if the title have changed.
        let current_title = Arc::new(Mutex::new(String::new()));

        // Playback volume
        let volume = Arc::new(Mutex::new(1.0));
        let volume_signal_id = None;

        let mut gstreamer_backend = Self {
            pipeline,
            recorderbin,
            current_title,
            volume,
            volume_signal_id,
            sender: gst_sender,
        };

        gstreamer_backend.setup_signals(app_sender);
        gstreamer_backend
    }

    fn setup_signals(&mut self, app_sender: Sender<Action>) {
        // We have to update the volume if we get changes from pulseaudio (pulsesink).
        // The user is able to control the volume from g-c-c.
        let (volume_sender, volume_receiver) = glib::MainContext::channel(glib::PRIORITY_LOW);
        let pulsesink = self.pipeline.get_by_name("pulsesink").unwrap();

        // We need to do message passing (sender/receiver) here, because gstreamer messages are
        // coming from a other thread (and app::Action enum is not thread safe).
        volume_receiver.attach(
            None,
            clone!(@strong app_sender => move |volume| {
                send!(app_sender, Action::PlaybackSetVolume(volume));
                glib::Continue(true)
            }),
        );

        // Update volume coming from pulseaudio / pulsesink
        self.volume_signal_id = Some(pulsesink.connect_notify(
            Some("volume"),
            clone!(@weak self.volume as old_volume, @strong volume_sender => move |element, _| {
                let new_volume: f64 = element.get_property("volume").unwrap().get().unwrap().unwrap();

                // We have to check if the values are the same. For some reason gstreamer sends us
                // slightly differents floats, so we round up here (only the the first two digits are
                // important for use here).
                let mut old_volume_locked = old_volume.lock().unwrap();
                let new_val = format!("{:.2}", old_volume_locked);
                let old_val = format!("{:.2}", old_volume_locked);

                if new_val != old_val {
                    send!(volume_sender, new_volume);
                    *old_volume_locked = new_volume;
                }
            }),
        ));

        // It's possible to mute the audio (!= 0.0) from pulseaudio side, so we should handle
        // this too by setting the volume to 0.0
        pulsesink.connect_notify(
            Some("mute"),
            clone!(@weak self.volume as old_volume, @strong volume_sender => move |element, _| {
                let mute: bool = element.get_property("mute").unwrap().get().unwrap().unwrap();
                let mut old_volume_locked = old_volume.lock().unwrap();
                if mute && *old_volume_locked != 0.0 {
                    send!(volume_sender, 0.0);
                    *old_volume_locked = 0.0;
                }
            }),
        );

        // listen for new pipeline / bus messages
        let bus = self.pipeline.get_bus().expect("Unable to get pipeline bus");
        bus.add_watch_local(
            clone!(@weak self.pipeline as pipeline, @strong self.sender as gst_sender, @weak self.current_title as current_title => @default-panic, move |_, message|{
                Self::parse_bus_message(pipeline, &message, gst_sender.clone(), current_title);
                Continue(true)
            }),
        )
        .unwrap();
    }

    pub fn set_state(&mut self, state: gstreamer::State) {
        debug!("Set playback state: {:?}", state);

        if state == gstreamer::State::Null {
            send!(self.sender, GstreamerMessage::PlaybackStateChanged(PlaybackState::Stopped));
        }

        let _ = self.pipeline.set_state(state);
    }

    pub fn set_volume(&self, volume: f64) {
        let pulsesink = self.pipeline.get_by_name("pulsesink").unwrap();

        // We need to block the signal, otherwise we risk creating a endless loop
        glib::signal::signal_handler_block(&pulsesink, &self.volume_signal_id.as_ref().unwrap());
        *self.volume.lock().unwrap() = volume;
        pulsesink.set_property("volume", &volume).unwrap();
        glib::signal::signal_handler_unblock(&pulsesink, &self.volume_signal_id.as_ref().unwrap());
    }

    pub fn new_source_uri(&mut self, source: &str) {
        debug!("Stop pipeline...");
        let _ = self.pipeline.set_state(State::Null);

        debug!("Set new source URI...");
        let uridecodebin = self.pipeline.get_by_name("uridecodebin").unwrap();
        uridecodebin.set_property("uri", &source).unwrap();

        debug!("Start pipeline...");
        let _ = self.pipeline.set_state(State::Playing);
    }

    pub fn start_recording(&mut self, path: PathBuf) {
        if self.is_recording() {
            warn!("Unable to start recording: Already recording");
            return;
        }
        debug!("Creating new recorderbin...");

        // Create actual recorderbin
        let description = "queue name=queue ! vorbisenc ! oggmux  ! filesink name=filesink";
        let recorderbin = gstreamer::parse_bin_from_description(description, true).expect("Unable to create recorderbin");
        recorderbin.set_property("message-forward", &true).unwrap();

        // We need to set an offset, otherwise the length of the recorded song would be wrong.
        // Get current clock time and calculate offset
        let offset = Self::calculate_pipeline_offset(&self.pipeline);
        let queue_srcpad = recorderbin.get_by_name("queue").unwrap().get_static_pad("src").unwrap();
        queue_srcpad.set_offset(offset);

        // Set recording path
        let filesink = recorderbin.get_by_name("filesink").unwrap();
        filesink.set_property("location", &(path.to_str().unwrap())).unwrap();

        // First try setting the recording bin to playing: if this fails we know this before it
        // potentially interferred with the other part of the pipeline
        recorderbin.set_state(gstreamer::State::Playing).expect("Failed to start recording");

        // Add new recorderbin to the pipeline
        self.pipeline.add(&recorderbin).expect("Unable to add recorderbin to pipeline");

        // Get our tee element by name, request a new source pad from it and then link that to our
        // recording bin to actually start receiving data
        let tee = self.pipeline.get_by_name("tee").unwrap();
        let tee_srcpad = tee.get_request_pad("src_%u").expect("Failed to request new pad from tee");
        let sinkpad = recorderbin.get_static_pad("sink").expect("Failed to get sink pad from recorderbin");

        // Link tee srcpad with the sinkpad of the recorderbin
        tee_srcpad.link(&sinkpad).expect("Unable to link tee srcpad with recorderbin sinkpad");

        *self.recorderbin.lock().unwrap() = Some(recorderbin);
        debug!("Started recording to {:?}", path);
    }

    pub fn stop_recording(&mut self, discard_data: bool) {
        let recorderbin = match self.recorderbin.lock().unwrap().take() {
            None => {
                warn!("Unable to stop recording: No recording running");
                return;
            }
            Some(bin) => bin,
        };

        debug!("Stop recording... (Discard recorded data: {:?})", &discard_data);

        // Get the source pad of the tee that is connected to the recorderbin
        let recorderbin_sinkpad = recorderbin.get_static_pad("sink").expect("Failed to get sink pad from recorderbin");
        let tee_srcpad = match recorderbin_sinkpad.get_peer() {
            Some(peer) => peer,
            None => return,
        };

        // Once the tee source pad is idle and we wouldn't interfere with any data flow, unlink the
        // tee and the recording bin and finalize the recording bin by sending it an end-of-stream
        // event
        //
        // Once the end-of-stream event is handled by the whole recording bin, we get an
        // end-of-stream message from it in the message handler and the shut down the recording bin
        // and remove it from the pipeline
        tee_srcpad.add_probe(
            PadProbeType::IDLE,
            clone!(@weak self.pipeline as pipeline => @default-panic, move |tee_srcpad, _| {
                // Get the parent of the tee source pad, i.e. the tee itself
                let tee = tee_srcpad
                    .get_parent()
                    .and_then(|parent| parent.downcast::<Element>().ok())
                    .expect("Failed to get tee source pad parent");

                // Unlink the tee source pad and then release it
                let _ = tee_srcpad.unlink(&recorderbin_sinkpad);
                tee.release_request_pad(tee_srcpad);

                if !discard_data {
                    // Asynchronously send the end-of-stream event to the sinkpad as this might block for a
                    // while and our closure here might've been called from the main UI thread
                    let recorderbin_sinkpad = recorderbin_sinkpad.clone();
                    recorderbin.call_async(move |_| {
                        recorderbin_sinkpad.send_event(Event::new_eos().build());
                        debug!("Sent EOS event to recorderbin sinkpad");
                    });
                }else{
                    Self::destroy_recorderbin(pipeline, recorderbin.clone());
                    debug!("Stopped recording.");
                }

                // Don't block the pad but remove the probe to let everything
                // continue as normal
                PadProbeReturn::Remove
            }),
        );
    }

    pub fn is_recording(&self) -> bool {
        self.recorderbin.lock().unwrap().is_some()
    }

    pub fn get_current_recording_duration(&self) -> i64 {
        let recorderbin: &Option<Bin> = &*self.recorderbin.lock().unwrap();
        if let Some(recorderbin) = recorderbin {
            let queue_srcpad = recorderbin.get_by_name("queue").unwrap().get_static_pad("src").unwrap();
            let offset = queue_srcpad.get_offset() / 1_000_000_000;

            let pipeline_time = self.pipeline.get_clock().expect("Could not get pipeline clock").get_time().nseconds().unwrap() as i64 / 1_000_000_000;
            let result = pipeline_time + offset + 1;

            return result;
        }

        warn!("No recording active, unable to get recording duration.");
        0
    }

    fn calculate_pipeline_offset(pipeline: &Pipeline) -> i64 {
        let clock_time = pipeline.get_clock().expect("Could not get pipeline clock").get_time().nseconds().unwrap() as i64;
        let base_time = pipeline.get_base_time().nseconds().expect("Could not get pipeline base time") as i64;
        -(clock_time - base_time)
    }

    fn destroy_recorderbin(pipeline: Pipeline, recorderbin: Bin) {
        // Ignore if the bin was not in the pipeline anymore for whatever
        // reason. It's not a problem
        let _ = pipeline.remove(&recorderbin);

        if let Err(err) = recorderbin.set_state(gstreamer::State::Null) {
            warn!("Failed to stop recording: {}", err);
        }
        debug!("Destroyed recorderbin.");
    }

    fn parse_bus_message(pipeline: Pipeline, message: &gstreamer::Message, sender: Sender<GstreamerMessage>, current_title: Arc<Mutex<String>>) {
        match message.view() {
            MessageView::Tag(tag) => {
                if let Some(t) = tag.get_tags().get::<gstreamer::tags::Title>() {
                    let new_title = t.get().unwrap().to_string();

                    // only send message if song title really have changed.
                    let mut current_title_locked = current_title.lock().unwrap();
                    if *current_title_locked != new_title {
                        *current_title_locked = new_title.clone();
                        send!(sender, GstreamerMessage::SongTitleChanged(new_title));
                    }
                }
            }
            MessageView::StateChanged(sc) => {
                let playback_state = match sc.get_current() {
                    gstreamer::State::Playing => PlaybackState::Playing,
                    gstreamer::State::Paused => PlaybackState::Playing,
                    gstreamer::State::Ready => PlaybackState::Playing,
                    _ => PlaybackState::Stopped,
                };

                send!(sender, GstreamerMessage::PlaybackStateChanged(playback_state));
            }
            MessageView::Element(element) => {
                // Catch the end-of-stream messages from the filesink
                let structure = element.get_structure().unwrap();
                if structure.get_name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap().unwrap();
                    if let MessageView::Eos(_) = &message.view() {
                        // Get recorderbin from message
                        let recorderbin = match message.get_src().and_then(|src| src.downcast::<Bin>().ok()) {
                            Some(src) => src,
                            None => return,
                        };

                        // And then asynchronously remove it and set its state to Null
                        pipeline.call_async(move |pipeline| {
                            Self::destroy_recorderbin(pipeline.clone(), recorderbin);
                            debug!("Stopped recording.");
                        });
                    }
                }
            }
            MessageView::Error(err) => {
                let msg = err.get_error().to_string();
                warn!("Gstreamer Error: {:?}", msg);
                send!(sender, GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(msg)));
            }
            _ => (),
        };
    }
}
