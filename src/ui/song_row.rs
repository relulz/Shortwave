// Shortwave - song_row.rs
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

use chrono::NaiveTime;
use glib::Sender;
use gtk::prelude::*;
use open;

use crate::app::Action;
use crate::audio::Song;

pub struct SongRow {
    pub widget: gtk::Box,
    song: Song,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl SongRow {
    pub fn new(sender: Sender<Action>, song: Song) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/song_row.ui");
        get_widget!(builder, gtk::Box, song_row);

        get_widget!(builder, gtk::Label, title_label);
        title_label.set_text(&song.title);
        title_label.set_tooltip_text(Some(song.title.as_str()));

        get_widget!(builder, gtk::Label, duration_label);
        duration_label.set_text(&Self::format_duration(song.duration.as_secs()));
        duration_label.set_tooltip_text(Some(Self::format_duration(song.duration.as_secs()).as_str()));

        let row = Self {
            widget: song_row,
            song,
            builder,
            sender,
        };

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        // save_button
        get_widget!(self.builder, gtk::Button, save_button);
        get_widget!(self.builder, gtk::Stack, button_stack);
        save_button.connect_clicked(
            clone!(@weak self.widget as widget, @weak button_stack, @strong self.song as song, @strong self.sender as sender => move |_| {
                send!(sender, Action::PlaybackSaveSong(song.clone()));

                // Show open button
                button_stack.set_visible_child_name("open");

                // Dim row
                let ctx = widget.get_style_context();
                ctx.add_class("dim-label");
            }),
        );

        // open_button
        get_widget!(self.builder, gtk::Button, open_button);
        open_button.connect_clicked(clone!(@strong self.song as song => move |_| {
            open::that(song.path.clone()).expect("Could not play song");
        }));
    }

    // stolen from gnome-podcasts
    // https://gitlab.gnome.org/haecker-felix/podcasts/blob/2f8a6a91f87d7fa335a954bbaf2f70694f32f6dd/podcasts-gtk/src/widgets/player.rs#L168
    fn format_duration(seconds: u64) -> String {
        debug!("Format duration (seconds): {}", &seconds);
        let time = NaiveTime::from_num_seconds_from_midnight(seconds as u32, 0);

        if seconds >= 3600 {
            time.format("%T").to_string()
        } else {
            time.format("%M∶%S").to_string()
        }
    }
}
