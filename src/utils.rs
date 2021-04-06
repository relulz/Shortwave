// Shortwave - utils.rs
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

use crate::api::StationMetadata;
use crate::i18n::*;

pub fn station_subtitle(metadata: StationMetadata) -> String {
    let mut string = if !metadata.country.is_empty() { metadata.country.to_string() } else { "".to_string() };

    if !metadata.state.is_empty() {
        string = format!("{} {}", string, metadata.state);
    }

    if string.is_empty() {
        string = ni18n_f("{} Vote", "{} Votes", metadata.votes as u32, &[&metadata.votes.to_string()]);
    } else {
        string = ni18n_f("{} · {} Vote", "{} · {} Votes", metadata.votes as u32, &[&string, &metadata.votes.to_string()]);
    }

    string
}
