use glib::Sender;
use indexmap::IndexMap;

use std::fs;
use std::path::PathBuf;

use crate::app::Action;
use crate::audio::Song;
use crate::path;
use crate::ui::SongListBox;

pub struct SongBackend {
    pub listbox: SongListBox,

    songs: IndexMap<String, Song>,
    save_count: usize,

    sender: Sender<Action>,
}

// songs: IndexMap<String, Song>,
// 1 - Song E    <- Oldest song, next to remove
// 2 ...
// 3 ...
// 4 ...
// 5 - Song A    <- Newest song

impl SongBackend {
    pub fn new(sender: Sender<Action>, save_count: usize) -> Self {
        let listbox = SongListBox::new(sender.clone());
        let songs = IndexMap::new();

        let song_backend = Self { listbox, songs, save_count, sender };
        song_backend
    }

    pub fn add_song(&mut self, song: Song) {
        // Check if song does not exist yet
        if self.songs.get(&song.title).is_none() {
            // Ensure max length
            if self.songs.len() >= self.save_count {
                // Get oldest song to remove
                let song = self.songs.get_index(0).unwrap().1.clone();
                self.remove_song(song.clone());
            }

            // Add song to indexmap & listbox
            self.listbox.add_song(song.clone());
            self.songs.insert(song.title.to_string(), song);
        } else {
            warn!("Song \"{}\" is already recorded", song.title);
        }
    }

    fn remove_song(&mut self, song: Song) {
        // Remove song from indexmap
        self.songs.shift_remove(&song.title);

        // Remove recorded data from disk
        fs::remove_file(&song.path).unwrap();

        // Removes the last row in song listbox
        self.listbox.remove_last_row();
    }

    pub fn save_song(&self, song: Song) {
        let mut dest_path = PathBuf::from(glib::get_user_special_dir(glib::UserDirectory::Music).unwrap());
        dest_path.push(song.path.file_name().unwrap());
        fs::copy(song.path, dest_path).unwrap();
    }

    pub fn delete_songs(&self) {
        let mut path = path::CACHE.clone();
        path.push("recording");

        // Just delete the whole recording dir.
        // It gets recreated automatically
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }
    }
}
