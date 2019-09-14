use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
    pub duration: Duration,
}

impl Song {
    pub fn new(title: &str, path: PathBuf, duration: Duration) -> Self {
        debug!("New song: \"{}\", {:?}", title, path);

        Self {
            title: title.to_string(),
            path,
            duration,
        }
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
