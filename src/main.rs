#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate strum_macros;

use gettextrs::*;
use std::env;

#[macro_use]
mod utils;

mod api;
mod audio;
mod database;
mod discover;
mod settings;
mod ui;

mod app;
mod config;
mod path;
mod static_resource;

use crate::app::App;

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    static_resource::init().expect("Failed to initialize the resource file.");

    // Initialize Gstreamer
    gstreamer::init().expect("Failed to initialize Gstreamer");

    // Initialize variables
    glib::set_application_name(config::NAME);
    glib::set_prgname(Some(&config::NAME.to_lowercase()));
    gtk::Window::set_default_icon_name(config::APP_ID);
    env::set_var("PULSE_PROP_application.icon_name", config::APP_ID);
    env::set_var("PULSE_PROP_application.name", config::NAME);

    // Setup translations
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("demo", config::LOCALEDIR);
    textdomain("shortwave");

    // Run app itself
    let app = App::new();
    app.run(app.clone());
}
