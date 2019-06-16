#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash)]
pub struct Station {
    pub id: String,
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub state: String,
    pub language: String,
    pub votes: String,
    pub negativevotes: String,
    pub lastchangetime: String,
    pub ip: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    pub lastcheckok: String,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    pub clickcount: String,
    pub clicktrend: String,
}

impl PartialEq for Station {
    fn eq(&self, other: &Station) -> bool {
        self.id == other.id
    }
}
