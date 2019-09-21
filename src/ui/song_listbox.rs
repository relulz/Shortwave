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
        let widget: gtk::Box = get_widget!(builder, "song_listbox");
        let listbox: gtk::ListBox = get_widget!(builder, "listbox");
        let stack: gtk::Stack = get_widget!(builder, "stack");

        Self { widget, listbox, stack, sender }
    }

    pub fn add_song(&mut self, song: Song) {
        let row = SongRow::new(self.sender.clone(), song.clone());
        self.listbox.insert(&row.widget, 0);

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
