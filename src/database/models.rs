use super::schema::*;
use crate::api::Station;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "library"]
pub struct StationIdentifier {
    pub id: Option<i32>, // Database ID
    pub station_id: i32, // Station ID
}

impl StationIdentifier {
    pub fn new(station: &Station) -> Self {
        StationIdentifier { id: None, station_id: station.id }
    }
}

impl PartialEq for StationIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.station_id == other.station_id
    }
}
