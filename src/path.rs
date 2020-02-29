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

use std::path::PathBuf;
use xdg;

lazy_static! {
    pub static ref BASE: xdg::BaseDirectories = { xdg::BaseDirectories::with_prefix(config::NAME).unwrap() };
    pub static ref DATA: PathBuf = { BASE.create_data_directory(BASE.get_data_home()).unwrap() };
    pub static ref CONFIG: PathBuf = { BASE.create_config_directory(BASE.get_config_home()).unwrap() };
    pub static ref CACHE: PathBuf = { BASE.create_cache_directory(BASE.get_cache_home()).unwrap() };
}
