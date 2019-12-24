use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::audio::Song;
use crate::ui::song_row::SongRow;

pub struct SongListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,
    stack: gtk::Stack,

    sender: Sender<Action>,
}

impl SongListBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_listbox.ui");
        get_widget!(builder, gtk::Box, song_listbox);
        get_widget!(builder, gtk::ListBox, listbox);
        get_widget!(builder, gtk::Stack, stack);

        Self {
            widget: song_listbox,
            listbox,
            stack,
            sender,
        }
    }

    pub fn add_song(&mut self, song: Song) {
        let row = SongRow::new(self.sender.clone(), song.clone());
        self.listbox.insert(&row, 0);

        self.update_stack();
    }

    pub fn remove_last_row(&self) {
        println!("\n\nRemove latest........\n\n");
        let mut children = self.listbox.get_children();
        println!("Listbox len: ");
        dbg!(children.len());

        let widget = children.pop().unwrap();

        self.listbox.remove(&widget);
        widget.destroy();

        self.update_stack();
    }

    fn update_stack(&self) {
        if self.listbox.get_children().len() != 0 {
            self.stack.set_visible_child_name("content");
        } else {
            self.stack.set_visible_child_name("empty");
        }
    }
}
