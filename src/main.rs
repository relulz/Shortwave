#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate glib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate matches;

mod api;
mod audio;
mod discover;
mod model;
mod widgets;

mod app;
mod config;
mod library;
mod static_resource;
mod window;
mod utils;

use crate::app::App;

fn main() {
    // Initialize logger
    pretty_env_logger::init();

    // Initialize GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    static_resource::init().expect("Failed to initialize the resource file.");

    // Initialize Gstreamer
    gstreamer::init().expect("Failed to initialize Gstreamer");

    // Run app itself
    let app = App::new();
    app.run(app.clone());
}
