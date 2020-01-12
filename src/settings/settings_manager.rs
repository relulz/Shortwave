use gio::prelude::*;

use crate::config;
use crate::settings::Key;

#[allow(deprecated)]
pub fn list_keys() {
    debug!("Settings values:");
    let settings = get_settings();
    let keys = settings.list_keys();

    for key in keys {
        let name = key.to_string();
        let value = settings.get_value(&name);
        debug!("  \"{}\" -> {}", name, value);
    }
}

pub fn create_action(key: Key) -> gio::Action {
    let settings = get_settings();
    settings.create_action(&key.to_string()).unwrap()
}

pub fn get_settings() -> gio::Settings {
    let app_id = config::APP_ID.trim_end_matches(".Devel");
    gio::Settings::new(app_id)
}

pub fn bind_property<P: IsA<glib::Object>>(key: Key, object: &P, property: &str) {
    let settings = get_settings();
    settings.bind(key.to_string().as_str(), object, property, gio::SettingsBindFlags::DEFAULT);
}

#[allow(dead_code)]
pub fn get_string(key: Key) -> String {
    let settings = get_settings();
    settings.get_string(&key.to_string()).unwrap().to_string()
}

#[allow(dead_code)]
pub fn set_string(key: Key, value: String) {
    let settings = get_settings();
    settings.set_string(&key.to_string(), &value).unwrap();
}

#[allow(dead_code)]
pub fn get_boolean(key: Key) -> bool {
    let settings = get_settings();
    settings.get_boolean(&key.to_string())
}

#[allow(dead_code)]
pub fn set_boolean(key: Key, value: bool) {
    let settings = get_settings();
    settings.set_boolean(&key.to_string(), value).unwrap();
}

#[allow(dead_code)]
pub fn get_integer(key: Key) -> i32 {
    let settings = get_settings();
    settings.get_int(&key.to_string())
}

#[allow(dead_code)]
pub fn set_integer(key: Key, value: i32) {
    let settings = get_settings();
    settings.set_int(&key.to_string(), value).unwrap();
}
