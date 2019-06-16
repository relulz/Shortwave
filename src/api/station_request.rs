#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct StationRequest {
    pub name: Option<String>,
    pub name_exact: Option<bool>,
    pub country: Option<String>,
    pub country_excat: Option<bool>,
    pub state: Option<String>,
    pub state_exact: Option<bool>,
    pub language: Option<String>,
    pub language_exact: Option<bool>,
    pub tag: Option<String>,
    pub tag_exact: Option<bool>,
    pub bitrate_min: Option<u32>,
    pub bitrate_max: Option<u32>,
    pub order: Option<String>,
    pub reverse: Option<bool>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl StationRequest {
    pub fn search_for_name(name: &str, limit: u32) -> Self {
        let mut search = Self::default();
        search.name = Some(name.to_string());
        search.limit = Some(limit);
        search
    }

    pub fn url_encode(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}
