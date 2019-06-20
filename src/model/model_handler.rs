use crate::api::Station;

pub trait ModelHandler {
    fn add_stations(&self, stations: Vec<Station>);
    fn remove_stations(&self, stations: Vec<Station>);
    fn clear(&self);
}
