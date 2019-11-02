mod backend;
mod controller;

pub use backend::GstreamerBackend;
pub use backend::SongBackend;
pub use controller::Controller;
pub use controller::GCastController;

mod gcast_discoverer;
mod player;
mod song;

pub use gcast_discoverer::GCastDevice;
pub use gcast_discoverer::GCastDiscoverer;
pub use player::PlaybackState;
pub use player::Player;
pub use song::Song;
