use diesel::connection::Connection;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use isahc::prelude::*;

use std::path::PathBuf;

use crate::api::Error;
use crate::database::models::StationIdentifier;

// It is possible to import Gradio stations in Shortwave.
// Gradio uses the deprecated radio-browser.info API, so we need to convert it first.

#[derive(Deserialize, Debug, Clone)]
pub struct GradioStation {
    pub id: String,
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub state: String,
    pub language: String,
    pub votes: String,
    pub negativevotes: String,
    pub lastchangetime: String,
    pub ip: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    pub lastcheckok: String,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    pub clickcount: String,
    pub clicktrend: String,
}

#[derive(QueryableByName, Debug)]
pub struct GradioStationID {
    #[sql_type = "Integer"]
    pub station_id: i32,
}

pub async fn read_database(path: PathBuf) -> Result<Vec<StationIdentifier>, Error> {
    // Establish connection to database
    let connection: diesel::SqliteConnection = Connection::establish(path.to_str().unwrap()).unwrap();

    // Read data from 'library' table
    let ids: Vec<GradioStationID> = diesel::sql_query("SELECT station_id FROM library;").load::<GradioStationID>(&connection).unwrap();

    let mut result = Vec::new();
    for id in ids {
        // Convert GradioIdentifier to Shortwave StationIdentifier
        let shortwave_id = StationIdentifier {
            id: None,
            stationuuid: id.station_id.clone().to_string(),
        };

        // We need to convert the StationIdentifier to UUID
        // For more details check
        // https://gitlab.gnome.org/World/Shortwave/issues/418
        match id2uuid(shortwave_id).await? {
            Some(shortwave_uuid) => result.push(shortwave_uuid),
            None => warn!("No UUID for ID \"{}\" found.", id.station_id.clone().to_string(),),
        };
    }
    Ok(result)
}

pub async fn id2uuid(identifier: StationIdentifier) -> Result<Option<StationIdentifier>, Error> {
    // We're going to use the old radio-browser.info API address here
    // to fetch the new UUID for a station.
    let url = &format!("https://www.radio-browser.info/webservice/json/stations/byid/{}", identifier.stationuuid);
    let data = isahc::get_async(url.to_string()).await?.text_async().await?;

    let s: Vec<GradioStation> = serde_json::from_str(data.as_str())?;
    if s.len() == 0 {
        return Ok(None);
    }
    let station = s[0].clone();

    debug!("Station ID {:?} -> UUID {:?}", identifier.stationuuid, station.stationuuid);
    let uuid = StationIdentifier {
        id: identifier.id,
        stationuuid: station.stationuuid,
    };
    Ok(Some(uuid))
}
