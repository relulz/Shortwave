use crate::database;
use crate::database::*;
use crate::diesel::prelude::*;

macro_rules! connect_db {
    () => {
        database::connection::get_connection().get().unwrap();
    };
}

pub fn get_station_identifiers() -> Result<Vec<StationIdentifier>, diesel::result::Error> {
    use crate::database::schema::library::dsl::*;
    let con = connect_db!();

    library.load::<StationIdentifier>(&con).map_err(From::from)
}

pub fn insert_station_identifier(identifier: StationIdentifier) -> Result<(), diesel::result::Error> {
    use crate::database::schema::library::dsl::*;
    let con = connect_db!();

    diesel::insert_into(library).values(identifier).execute(&*con).map_err(From::from).map(|_| ())
}

pub fn delete_station_identifier(identifier: StationIdentifier) -> Result<(), diesel::result::Error> {
    use crate::database::schema::library::dsl::*;
    let con = connect_db!();

    diesel::delete(library.filter(stationuuid.eq(identifier.stationuuid))).execute(&*con).map_err(From::from).map(|_| ())
}
