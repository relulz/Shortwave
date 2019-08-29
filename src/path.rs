use crate::config;

use std::path::PathBuf;
use xdg;

lazy_static! {
    pub static ref BASE: xdg::BaseDirectories = { xdg::BaseDirectories::with_prefix(config::NAME).unwrap() };
    pub static ref DATA: PathBuf = { BASE.create_data_directory(BASE.get_data_home()).unwrap() };
    pub static ref CONFIG: PathBuf = { BASE.create_config_directory(BASE.get_config_home()).unwrap() };
    pub static ref CACHE: PathBuf = { BASE.create_cache_directory(BASE.get_cache_home()).unwrap() };
}
