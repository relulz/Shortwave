use super::schema::*;
use crate::api::Station;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "library"]
pub struct StationIdentifier {
    pub id: Option<i32>,     // Database ID
    pub stationuuid: String, // Station UUID
}

impl StationIdentifier {
    pub fn from_station(station: &Station) -> Self {
        StationIdentifier {
            id: None,
            stationuuid: station.stationuuid.clone(),
        }
    }
}

impl PartialEq for StationIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.stationuuid == other.stationuuid
    }
}
