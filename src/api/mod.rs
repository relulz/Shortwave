static STATION_SEARCH: &'static str = "json/stations/search";
static STATION_BY_ID: &'static str = "json/stations/byid/";
static PLAYABLE_STATION_URL: &'static str = "json/url/";

mod client;
mod error;
mod object;
mod station;
mod station_request;
mod station_url;

pub use client::Client;
pub use error::Error;
pub use object::Object;
pub use station::Station;
pub use station_request::StationRequest;
pub use station_url::StationUrl;
