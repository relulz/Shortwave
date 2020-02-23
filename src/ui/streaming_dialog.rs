use glib::Sender;
use gtk::prelude::*;

use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use crate::app::Action;
use crate::audio::GCastDiscoverer;
use crate::audio::GCastDiscovererMessage;
use crate::utils;

pub struct StreamingDialog {
    pub widget: libhandy::Dialog,
    gcd: Rc<GCastDiscoverer>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StreamingDialog {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui");
        get_widget!(builder, libhandy::Dialog, streaming_dialog);

        // Setup Google Cast discoverer
        let gcd_t = GCastDiscoverer::new();
        let gcd = Rc::new(gcd_t.0);
        let gcd_receiver = gcd_t.1;

        let sd = Self {
            widget: streaming_dialog,
            gcd,
            builder,
            sender,
        };

        get_widget!(sd.builder, gtk::Stack, stream_stack);
        get_widget!(sd.builder, gtk::ListBox, devices_listbox);
        get_widget!(sd.builder, gtk::Button, connect_button);
        get_widget!(sd.builder, gtk::Revealer, loading_revealer);
        gcd_receiver.attach(None, move |message| {
            match message {
                GCastDiscovererMessage::DiscoverStarted => {
                    utils::remove_all_items(&devices_listbox);
                    stream_stack.set_visible_child_name("loading");
                    loading_revealer.set_reveal_child(true);
                }
                GCastDiscovererMessage::DiscoverEnded => {
                    if devices_listbox.get_children().is_empty() {
                        stream_stack.set_visible_child_name("no-devices");
                    } else {
                        stream_stack.set_visible_child_name("results");
                    }
                    loading_revealer.set_reveal_child(false);
                }
                GCastDiscovererMessage::FoundDevice(device) => {
                    stream_stack.set_visible_child_name("results");
                    connect_button.set_sensitive(true);

                    let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui");
                    get_widget!(builder, gtk::ListBoxRow, device_row);
                    get_widget!(builder, gtk::Label, name_label);
                    get_widget!(builder, gtk::Label, ip_label);

                    name_label.set_text(&device.name);
                    ip_label.set_text(&device.ip.to_string());
                    device_row.show_all();

                    devices_listbox.add(&device_row);
                }
            }

            glib::source::Continue(true)
        });

        sd.setup_signals();
        sd
    }

    pub fn show(&self) {
        let application = self.builder.get_application().unwrap();
        let window = application.get_active_window().unwrap();
        self.widget.set_transient_for(Some(&window));

        self.widget.set_visible(true);
        self.widget.show();

        self.gcd.start_discover();
    }

    fn setup_signals(&self) {
        // retry_button
        let gcd = self.gcd.clone();
        get_widget!(self.builder, gtk::Button, retry_button);
        retry_button.connect_clicked(move |_| {
            gcd.start_discover();
        });

        // cancel_button
        let widget = self.widget.clone();
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(move |_| {
            widget.set_visible(false);
            widget.hide();
        });

        // connect_button
        get_widget!(self.builder, gtk::ListBox, devices_listbox);
        get_widget!(self.builder, gtk::Button, connect_button);
        let widget = self.widget.clone();
        let gcd = self.gcd.clone();
        let sender = self.sender.clone();
        connect_button.connect_clicked(move |_| {
            if let Some(active_row) = devices_listbox.get_selected_row() {
                // Very hackish way to get the selected ip address
                let box1: gtk::Box = active_row.get_children()[0].clone().downcast().unwrap();
                let box2: gtk::Box = box1.get_children()[0].clone().downcast().unwrap();
                let ip_label: gtk::Label = box2.get_children()[1].clone().downcast().unwrap();
                let ip_addr: IpAddr = IpAddr::from_str(ip_label.get_text().unwrap().to_string().as_str()).unwrap();

                // Get GCastDevice
                let device = gcd.get_device_by_ip_addr(ip_addr).unwrap();
                send!(sender, Action::PlaybackConnectGCastDevice(device));
                widget.set_visible(false);
                widget.hide();
            }
        });

        // hide on delete
        self.widget.connect_delete_event(|widget, _| {
            widget.hide_on_delete();
            widget.hide();
            glib::signal::Inhibit(true)
        });
    }
}
