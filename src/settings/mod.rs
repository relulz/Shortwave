use gio::prelude::*;
use inflector::Inflector;

use crate::config;

#[derive(Debug)]
pub enum Key {
    ApiServer,
    DarkMode,
}

impl Key{
    pub fn to_string(&self) -> String{
        let string = format!("{:?}", self);
        string.to_kebab_case()
    }
}

fn get_settings() -> gio::Settings{
    let app_id = config::APP_ID.trim_end_matches(".Devel");
    gio::Settings::new(app_id)
}

pub fn get_string(key: Key) -> String {
    let settings = get_settings();
    settings.get_string(&key.to_string()).unwrap().to_string()
}

pub fn set_string(key: Key, value: String) {
    let settings = get_settings();
    settings.set_string(&key.to_string(), &value);
}

pub fn get_boolean(key: Key) -> bool {
    let settings = get_settings();
    settings.get_boolean(&key.to_string())
}

pub fn set_boolean(key: Key, value: bool) {
    let settings = get_settings();
    settings.set_boolean(&key.to_string(), value);
}

pub fn get_integer(key: Key) -> i32 {
    let settings = get_settings();
    settings.get_int(&key.to_string())
}

pub fn set_integer(key: Key, value: i32) {
    let settings = get_settings();
    settings.set_int(&key.to_string(), value);
}
