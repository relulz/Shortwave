use mdns::{Record, RecordKind};
use glib::{Sender, Receiver};

use std::net::IpAddr;
use std::thread;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct GCastDevice{
    pub id: String,
    pub ip: IpAddr,
    pub name: String,
}

pub struct GCastDiscoverer{
    sender: Sender<GCastDevice>,
    known_devices: Arc<Mutex<Vec<GCastDevice>>>,
}

impl GCastDiscoverer{
    pub fn new() -> (Self, Receiver<GCastDevice>){
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let known_devices = Arc::new(Mutex::new(Vec::new()));

        let gcd = Self { sender, known_devices };
        (gcd, receiver)
    }

    pub fn start_discover(&self){
        debug!("Start searching for google cast devices...");
        let known_devices = self.known_devices.clone();
        let sender = self.sender.clone();

        thread::spawn(move || {
            for response in mdns::discover::all("_googlecast._tcp.local").unwrap() {
                let response = response.unwrap();

                let known_devices = known_devices.clone();
                let sender = sender.clone();
                Self::get_device(response).map(move |device|{
                    if !known_devices.lock().unwrap().contains(&device){
                        debug!("Found new google cast device!");
                        debug!("{:?}", device);
                        known_devices.lock().unwrap().insert(0, device.clone());
                        sender.send(device).unwrap();
                    }
                });
            }
        });
    }

    pub fn get_device_by_ip_addr(&self, ip: IpAddr) -> Option<GCastDevice>{
        for device in self.known_devices.lock().unwrap().iter(){
            if device.ip == ip {
                return Some(device.clone());
            }
        }
        None
    }

    fn get_device(response: mdns::Response) -> Option<GCastDevice> {
        let mut values: HashMap<String, String> = HashMap::new();

        let addr = response.records().filter_map(Self::to_ip_addr).next();
        if addr == None{
            debug!("Cast device does not advertise address.");
            return None;
        }

        // To get the values, we need to iterate the additional records and check the TXT kind
        for record in response.additional{
            // Check if record kind is TXT
            if let mdns::RecordKind::TXT(v) = record.kind {
                // Iterate TXT values
                for value in v {
                    let tmp = value.split("=").collect::<Vec<&str>>();
                    values.insert(tmp[0].to_string(), tmp[1].to_string());
                }
                break;
            }
        }

        let device = GCastDevice{
            id: values.get("id").unwrap().to_string(),
            ip: addr.unwrap(),
            name: values.get("fn").unwrap().to_string()
        };

        Some(device)
    }

    fn to_ip_addr(record: &Record) -> Option<IpAddr> {
        match record.kind {
            RecordKind::A(addr) => Some(addr.into()),
            RecordKind::AAAA(addr) => Some(addr.into()),
            _ => None,
        }
    }
}
