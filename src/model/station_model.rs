use crate::api::Station;
use crate::model::ModelHandler;
use crate::widgets::StationFlowBox;

#[derive(Clone, Debug)]
pub enum Sorting {
    Name,
    Language,
    Country,
    State,
    Codec,
    Votes,
    Bitrate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}

pub struct StationModel {
    data: Vec<Station>,
    sorting: Sorting,
    order: Order,

    handler: Vec<Box<ModelHandler>>,
}

impl StationModel {
    pub fn new() -> Self {
        let data = Vec::new();
        let sorting = Sorting::Name;
        let order = Order::Ascending;
        let handler: Vec<Box<ModelHandler>> = Vec::new();

        Self { data, sorting, order, handler }
    }

    pub fn add_stations(&mut self, stations: Vec<Station>) {
        for station in &stations {
            if !self.data.contains(&station) {
                self.data.push(station.clone());
            }
        }

        for h in &*self.handler {
            h.add_stations(stations.clone());
        }
    }

    pub fn remove_stations(&mut self, stations: Vec<Station>) {
        for station in &stations {
            let index = self.data.iter().position(|s| s == station).unwrap();
            self.data.remove(index);
        }

        for h in &*self.handler {
            h.remove_stations(stations.clone());
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();

        for h in &*self.handler {
            h.clear();
        }
    }

    pub fn set_sorting(&mut self, sorting: Sorting, order: Order) {
        self.sorting = sorting.clone();
        self.order = order.clone();
    }

    pub fn export(&self) -> Vec<Station> {
        self.data.clone()
    }

    /// Bind to a struct which implements the trait ModelHandler
    pub fn bind(&mut self, handler: Box<ModelHandler>) {
        self.handler.push(handler);
    }

    fn station_cmp(station_a: &Station, station_b: &Station, sorting: Sorting, order: Order) -> std::cmp::Ordering {
        if order == Order::Descending {
            let (station_a, station_b) = (station_b, station_a);
        }

        match sorting {
            Sorting::Name => station_a.name.cmp(&station_b.name),
            Sorting::Language => station_a.language.cmp(&station_b.language),
            Sorting::Country => station_a.country.cmp(&station_b.country),
            Sorting::State => station_a.state.cmp(&station_b.state),
            Sorting::Codec => station_a.codec.cmp(&station_b.codec),
            Sorting::Votes => station_a.votes.parse::<i32>().unwrap().cmp(&station_b.votes.parse::<i32>().unwrap()),
            Sorting::Bitrate => station_a.bitrate.parse::<i32>().unwrap().cmp(&station_b.bitrate.parse::<i32>().unwrap()),
        }
    }
}
