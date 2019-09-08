mod mini_controller;
mod mpris_controller;
mod sidebar_controller;

pub use mini_controller::MiniController;
pub use mpris_controller::MprisController;
pub use sidebar_controller::SidebarController;

use crate::api::Station;
use crate::audio::PlaybackState;

pub trait Controller {
    fn set_station(&self, station: Station);
    fn set_playback_state(&self, playback_state: &PlaybackState);
    fn set_song_title(&self, title: &str);
}
