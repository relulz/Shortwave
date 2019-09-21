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
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_row.ui");
        let song_row: gtk::Box = get_widget!(builder, "song_row");

        let title_label: gtk::Label = get_widget!(builder, "title_label");
        title_label.set_text(&song.title);
        title_label.set_tooltip_text(Some(song.title.as_str()));
        let duration_label: gtk::Label = get_widget!(builder, "duration_label");
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
        let sender = self.sender.clone();
        let song = self.song.clone();
        let widget = self.widget.clone();
        let button_stack: gtk::Stack = get_widget!(self.builder, "button_stack");
        let save_button: gtk::Button = get_widget!(self.builder, "save_button");
        save_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackSaveSong(song.clone())).unwrap();

            // Show open button
            button_stack.set_visible_child_name("open");

            // Dim row
            let ctx = widget.get_style_context();
            ctx.add_class("dim-label");
        });

        let song = self.song.clone();
        let open_button: gtk::Button = get_widget!(self.builder, "open_button");
        open_button.connect_clicked(move |_| {
            open::that(song.path.clone()).expect("Could not play song");
        });
    }

    // stolen from gnome-podcasts
    // https://gitlab.gnome.org/haecker-felix/podcasts/blob/2f8a6a91f87d7fa335a954bbaf2f70694f32f6dd/podcasts-gtk/src/widgets/player.rs#L168
    fn format_duration(seconds: u64) -> String {
        let time = NaiveTime::from_num_seconds_from_midnight(seconds as u32, 0);

        if seconds >= 3600 {
            time.format("%T").to_string()
        } else {
            time.format("%M∶%S").to_string()
        }
    }
}
