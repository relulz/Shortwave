use diesel::connection::Connection;
use diesel::prelude::*;
use diesel::sql_types::Integer;

use std::path::PathBuf;

use crate::database::models::StationIdentifier;

// We're still using the old Gradio DB format for importing / exporting stations.
// So we can ensure that we can transfer data from Gradio to Shortwave, and and vice versa.

#[derive(QueryableByName, Debug)]
pub struct GradioStationID {
    #[sql_type = "Integer"]
    pub station_id: i32,
}

pub fn read_database(path: PathBuf) -> Vec<StationIdentifier>{
    // Establish connection to database
    let connection: diesel::SqliteConnection = Connection::establish(path.to_str().unwrap()).unwrap(); //TODO: don't unwrap

    // Read data from 'library' table
    let ids = diesel::sql_query("SELECT station_id FROM library;").load::<GradioStationID>(&connection).unwrap();
    dbg!(&ids);

    // Convert GradioIdentifier to Shortwave StationIdentifier
    let mut result = Vec::new();
    for id in ids {
        let sid = StationIdentifier{
            id: None,
            station_id: id.station_id.clone(),
        };
        result.push(sid);
    }
    result
}
