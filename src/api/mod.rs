static STATION_SEARCH: &str = "json/stations/search";
static STATION_BY_UUID: &str = "json/stations/byuuid/";

mod client;
mod error;
mod favicon_downloader;
mod object;
mod station;
mod station_request;
mod station_url;

pub use client::Client;
pub use error::Error;
pub use favicon_downloader::FaviconDownloader;
pub use object::Object;
pub use station::Station;
pub use station_request::StationRequest;
pub use station_url::StationUrl;
