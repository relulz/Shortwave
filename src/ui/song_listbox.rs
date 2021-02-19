// Shortwave - song_listbox.rs
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

use glib::Sender;
use gtk::glib;
use gtk::prelude::*;
use open;

use crate::app::Action;
use crate::audio::Song;
use crate::ui::song_row::SongRow;

pub struct SongListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,
    stack: gtk::Stack,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl SongListBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/song_listbox.ui");
        get_widget!(builder, gtk::Box, song_listbox);
        get_widget!(builder, gtk::ListBox, listbox);
        get_widget!(builder, gtk::Stack, stack);

        let listbox = Self {
            widget: song_listbox,
            listbox,
            stack,
            builder,
            sender,
        };

        listbox.setup_signals();
        listbox
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, open_music_folder_button);
        open_music_folder_button.connect_clicked(|_| {
            open::that(glib::get_user_special_dir(glib::UserDirectory::Music)).expect("Unable to open music folder");
        });
    }

    pub fn add_song(&mut self, song: Song) {
        let row = SongRow::new(self.sender.clone(), song);
        self.listbox.insert(&row.widget, 0);

        self.update_stack();
    }

    pub fn remove_last_row(&self) {
        if let Some(child) = self.listbox.get_last_child() {
            self.listbox.remove(&child);
        }

        self.update_stack();
    }

    fn update_stack(&self) {
        if !self.listbox.get_last_child().is_none() {
            self.stack.set_visible_child_name("content");
        } else {
            self.stack.set_visible_child_name("empty");
        }
    }
}
