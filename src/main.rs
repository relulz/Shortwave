#[macro_use]
extern crate log;
extern crate pretty_env_logger;
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
#[macro_use]
extern crate glib;
#[macro_use]
extern crate gtk_macros;

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

use crate::app::SwApplication;

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    // Initialize Gstreamer
    gstreamer::init().expect("Failed to initialize Gstreamer");

    // Initialize variables
    glib::set_application_name(config::NAME);
    glib::set_prgname(Some(&config::PKGNAME));
    gtk::Window::set_default_icon_name(config::APP_ID);
    env::set_var("PULSE_PROP_application.icon_name", config::APP_ID);
    env::set_var("PULSE_PROP_application.name", config::NAME);

    // Setup translations
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::PKGNAME, config::LOCALEDIR);
    textdomain(config::PKGNAME);

    // Load app resources
    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + &format!("/{}.gresource", config::APP_ID)).expect("Could not load resources");
    gio::resources_register(&res);

    // Run app itself
    SwApplication::run();
}
