// Shortwave - streaming_dialog.rs
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
use gtk::prelude::*;

use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use crate::app::{Action, SwApplication};
use crate::audio::GCastDiscoverer;
use crate::audio::GCastDiscovererMessage;
use crate::utils;

pub struct StreamingDialog {
    pub widget: gtk::Dialog,
    gcd: Rc<GCastDiscoverer>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StreamingDialog {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui");
        get_widget!(builder, gtk::Dialog, streaming_dialog);

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

        gcd_receiver.attach(
            None,
            clone!(@weak sd.builder as builder => @default-panic, move |message| {
                get_widget!(builder, gtk::Stack, stream_stack);
                get_widget!(builder, gtk::ListBox, devices_listbox);
                get_widget!(builder, gtk::Button, connect_button);
                get_widget!(builder, gtk::Revealer, loading_revealer);

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

                        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui");
                        get_widget!(builder, gtk::ListBoxRow, device_row);
                        get_widget!(builder, gtk::Label, name_label);
                        get_widget!(builder, gtk::Label, ip_label);

                        name_label.set_text(&device.name);
                        ip_label.set_text(&device.ip.to_string());

                        devices_listbox.append(&device_row);
                    }
                }

                glib::source::Continue(true)
            }),
        );

        sd.setup_signals();
        sd
    }

    pub fn show(&self) {
        let window = gio::Application::get_default().unwrap().downcast_ref::<SwApplication>().unwrap().get_active_window().unwrap();
        self.widget.set_transient_for(Some(&window));

        self.widget.set_visible(true);
        self.widget.show();

        self.gcd.start_discover();
    }

    fn setup_signals(&self) {
        // retry_button
        get_widget!(self.builder, gtk::Button, retry_button);
        retry_button.connect_clicked(clone!(@weak self.gcd as gcd => move |_| {
            gcd.start_discover();
        }));

        // cancel_button
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(clone!(@weak self.widget as widget => move |_| {
            widget.set_visible(false);
            widget.hide();
        }));

        // connect_button
        get_widget!(self.builder, gtk::Button, connect_button);
        connect_button.connect_clicked(clone!(@weak self.builder as builder, @weak self.gcd as gcd, @strong self.sender as sender => move |_| {
            get_widget!(builder, gtk::ListBox, devices_listbox);
            get_widget!(builder, gtk::Dialog, streaming_dialog);

            if let Some(active_row) = devices_listbox.get_selected_row() {
                // Very hackish way to get the selected ip address
                let box1: gtk::Box = active_row.get_first_child().unwrap().clone().downcast().unwrap();
                let box2: gtk::Box = box1.get_first_child().unwrap().clone().downcast().unwrap();
                let ip_label: gtk::Label = box2.get_last_child().unwrap().clone().downcast().unwrap();
                let ip_addr: IpAddr = IpAddr::from_str(ip_label.get_text().to_string().as_str()).unwrap();

                // Get GCastDevice
                let device = gcd.get_device_by_ip_addr(ip_addr).unwrap();
                send!(sender, Action::PlaybackConnectGCastDevice(device));
                streaming_dialog.set_visible(false);
                streaming_dialog.hide();
            }
        }));

        // hide on delete
        self.widget.connect_close_request(|widget| {
            widget.hide();
            glib::signal::Inhibit(true)
        });
    }
}
