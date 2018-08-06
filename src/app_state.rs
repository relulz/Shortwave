use rustio::station::Station;
use library::NewLibrary;
use audioplayer::PlaybackState;

use mdl::model::Model;

// The AppState contains all important data that must
// be available in the complete application

#[derive(Serialize, Deserialize, Debug)]
pub struct AppState{
    pub library: NewLibrary,

    // Audio playback (ap)           Signal:
    pub ap_station: Option<Station>, // ap-station
    pub ap_title: Option<String>,    // ap-title
    pub ap_state: PlaybackState,     // ap-playback
}

impl Model for AppState {
    fn key(&self) -> String { "app".to_string() }
}

impl AppState{
    pub fn new() -> Self {
        let library = NewLibrary::new();

        let ap_station = None;
        let ap_title = None;
        let ap_state = PlaybackState::Stopped;

        AppState{
            library,
            ap_station,
            ap_title,
            ap_state,
        }
    }
}
