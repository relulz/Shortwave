// Shortwave - gcast_discoverer.rs
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
use mdns::{Record, RecordKind};

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
pub struct GCastDevice {
    pub id: String,
    pub ip: IpAddr,
    pub name: String,
}

pub enum GCastDiscovererMessage {
    DiscoverStarted,
    DiscoverEnded,
    FoundDevice(GCastDevice),
}

pub struct GCastDiscoverer {
    sender: Sender<GCastDiscovererMessage>,
    known_devices: Arc<Mutex<Vec<GCastDevice>>>,
}

impl GCastDiscoverer {
    pub fn new() -> (Self, Receiver<GCastDiscovererMessage>) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let known_devices = Arc::new(Mutex::new(Vec::new()));

        let gcd = Self { sender, known_devices };
        (gcd, receiver)
    }

    pub fn start_discover(&self) {
        let known_devices = self.known_devices.clone();
        let sender = self.sender.clone();

        thread::spawn(move || {
            // Reset previous found devices
            known_devices.lock().unwrap().clear();

            debug!("Start discovering for google cast devices...");
            send!(sender, GCastDiscovererMessage::DiscoverStarted);

            let discovery = mdns::discover::all("_googlecast._tcp.local").unwrap();
            let discovery = discovery.timeout(std::time::Duration::from_secs(10));

            for response in discovery {
                let response = response.unwrap();

                let known_devices = known_devices.clone();
                let sender = sender.clone();
                if let Some(device) = Self::get_device(response) {
                    if !known_devices.lock().unwrap().contains(&device) {
                        debug!("Found new google cast device!");
                        debug!("{:?}", device);
                        known_devices.lock().unwrap().insert(0, device.clone());
                        send!(sender, GCastDiscovererMessage::FoundDevice(device));
                    }
                }
            }

            send!(sender, GCastDiscovererMessage::DiscoverEnded);
            debug!("GCast discovery ended.")
        });
    }

    pub fn get_device_by_ip_addr(&self, ip: IpAddr) -> Option<GCastDevice> {
        for device in self.known_devices.lock().unwrap().iter() {
            if device.ip == ip {
                return Some(device.clone());
            }
        }
        None
    }

    fn get_device(response: mdns::Response) -> Option<GCastDevice> {
        let mut values: HashMap<String, String> = HashMap::new();

        let addr = response.records().filter_map(Self::record_to_ip_addr).next();
        if addr == None {
            debug!("Cast device does not advertise address.");
            return None;
        }

        // To get the values, we need to iterate the additional records and check the TXT kind
        for record in response.additional {
            // Check if record kind is TXT
            if let mdns::RecordKind::TXT(v) = record.kind {
                // Iterate TXT values
                for value in v {
                    let tmp = value.split('=').collect::<Vec<&str>>();
                    values.insert(tmp[0].to_string(), tmp[1].to_string());
                }
                break;
            }
        }

        let device = GCastDevice {
            id: values.get("id").unwrap().to_string(),
            ip: addr.unwrap(),
            name: values.get("fn").unwrap().to_string(),
        };

        Some(device)
    }

    fn record_to_ip_addr(record: &Record) -> Option<IpAddr> {
        match record.kind {
            RecordKind::A(addr) => Some(addr.into()),
            RecordKind::AAAA(addr) => Some(addr.into()),
            _ => None,
        }
    }
}
