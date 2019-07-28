use chrono::NaiveTime;
use glib::Sender;
use gtk::prelude::*;
use open;

use std::path::PathBuf;

use crate::app::Action;
use crate::audio::Song;

pub struct SongRow {
    pub widget: gtk::Box,
    song: Song,

    builder: gtk::Builder,
}

impl SongRow {
    pub fn new(_sender: Sender<Action>, song: Song) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_row.ui");
        let song_row: gtk::Box = builder.get_object("song_row").unwrap();

        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        title_label.set_text(&song.title);
        title_label.set_tooltip_text(Some(song.title.as_str()));
        let duration_label: gtk::Label = builder.get_object("duration_label").unwrap();
        duration_label.set_text(&Self::format_duration(song.duration.as_secs()));
        duration_label.set_tooltip_text(Some(Self::format_duration(song.duration.as_secs()).as_str()));

        let row = Self { widget: song_row, song, builder };

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        let song = self.song.clone();
        let button_stack: gtk::Stack = self.builder.get_object("button_stack").unwrap();
        let save_button: gtk::Button = self.builder.get_object("save_button").unwrap();
        let duration_label: gtk::Label = self.builder.get_object("duration_label").unwrap();
        save_button.connect_clicked(move |_| {
            let mut path = PathBuf::from(glib::get_user_special_dir(glib::UserDirectory::Music).unwrap());
            path.push(&Song::simplify_title(song.title.clone()));
            match song.save_as(path) {
                Ok(()) => {
                    duration_label.set_text("Saved");
                    button_stack.set_visible_child_name("open");
                }
                Err(err) => duration_label.set_text(&err.to_string()),
            };
        });

        let song = self.song.clone();
        let open_button: gtk::Button = self.builder.get_object("open_button").unwrap();
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
