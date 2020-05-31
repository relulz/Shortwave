// Shortwave - gradio_db.rs
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

use diesel::connection::Connection;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use glib::Sender;
use isahc::prelude::*;
use url::Url;

use std::path::PathBuf;

use crate::api::{Client, Error};
use crate::app::Action;
use crate::database::models::StationIdentifier;
use crate::i18n::*;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;

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
    debug!("Read database: {:?}", path);

    // Establish connection to database
    let connection: diesel::SqliteConnection = Connection::establish(path.to_str().unwrap()).unwrap();

    // Read data from 'library' table
    let ids: Vec<GradioStationID> = diesel::sql_query("SELECT station_id FROM library;").load::<GradioStationID>(&connection)?;

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
    if s.is_empty() {
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

pub async fn import_database(path: PathBuf, sender: Sender<Action>) -> Result<(), Error> {
    debug!("Import path: {:?}", path);

    // Get station identifiers
    let spinner_notification = Notification::new_spinner(&i18n("Converting data…"));
    send!(sender, Action::ViewShowNotification(spinner_notification.clone()));
    let ids = read_database(path).await?;
    spinner_notification.hide();

    // Get actual stations from identifiers
    let message = ni18n_f("Importing {} station…", "Importing {} stations…", ids.len() as u32, &[&ids.len().to_string()]);
    let spinner_notification = Notification::new_spinner(&message);
    send!(sender, Action::ViewShowNotification(spinner_notification.clone()));

    let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());
    let sender = sender.clone();
    let mut stations = Vec::new();
    for id in ids {
        let station = client.clone().get_station_by_identifier(id).await?;
        stations.insert(0, station);
    }

    spinner_notification.hide();
    send!(sender, Action::LibraryAddStations(stations.clone()));
    let message = ni18n_f("Imported {} station!", "Imported {} stations!", stations.len() as u32, &[&stations.len().to_string()]);
    let notification = Notification::new_info(&message);
    send!(sender, Action::ViewShowNotification(notification));

    Ok(())
}
