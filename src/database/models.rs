// Shortwave - models.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
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

use super::schema::*;
use crate::api::SwStation;

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "library"]
pub struct StationIdentifier {
    pub id: Option<i32>,     // Database ID
    pub stationuuid: String, // Station UUID
}

impl StationIdentifier {
    pub fn from_station(station: &SwStation) -> Self {
        StationIdentifier {
            id: None,
            stationuuid: station.metadata().stationuuid.clone(),
        }
    }
    pub fn from_uuid(uuid: String) -> Self {
        StationIdentifier { id: None, stationuuid: uuid }
    }
}

impl PartialEq for StationIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.stationuuid == other.stationuuid
    }
}
