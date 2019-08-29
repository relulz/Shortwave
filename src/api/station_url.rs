#[derive(Serialize, Deserialize, Clone)]
pub struct StationUrl {
    pub ok: String,
    pub message: String,
    pub id: String,
    pub name: String,
    pub url: String,
}
