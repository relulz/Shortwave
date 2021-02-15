// Shortwave - path.rs
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

use crate::config;

use std::fs;
use std::path::PathBuf;

lazy_static! {
    pub static ref DATA: PathBuf = {
        let mut path = glib::get_user_data_dir();
        path.push(config::NAME);
        path
    };
    pub static ref CONFIG: PathBuf = {
        let mut path = glib::get_user_config_dir();
        path.push(config::NAME);
        path
    };
    pub static ref CACHE: PathBuf = {
        let mut path = glib::get_user_cache_dir();
        path.push(config::NAME);
        path
    };
}

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(DATA.to_owned())?;
    fs::create_dir_all(CONFIG.to_owned())?;
    fs::create_dir_all(CACHE.to_owned())?;
    Ok(())
}
