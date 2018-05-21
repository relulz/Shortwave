extern crate gio;
extern crate gtk;
extern crate gdk;
use gio::{ApplicationExt, ApplicationExtManual};
use gtk::prelude::*;

use rustio::{audioplayer::AudioPlayer, client::Client};

use std::cell::RefCell;
use std::rc::Rc;
use std::io::Read;

use page::library_page::LibraryPage;
use page::search_page::SearchPage;
use page::Page;

use library::Library;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use rustio::station::Station;
use std::fs::File;

pub enum Action {
    /* Audio Playback Actions */
    PlaybackStart,
    PlaybackStop,
    PlaybackSetStation(Station),

    /* Library Actions */
    LibraryAdd(String),
    LibraryRemove(String),
}

pub struct GradioApp {
    player: AudioPlayer,
    library: Library,

    receiver: Receiver<Action>,
    sender: Sender<Action>,

    builder: gtk::Builder,
    gtk_app: gtk::Application,
    window: gtk::ApplicationWindow,

    page_stack: gtk::Stack,
    library_page: LibraryPage,
    search_page: SearchPage,

    playerbar: gtk::ActionBar,
    station_title: gtk::Label,
    station_subtitle: gtk::Label,
}

impl GradioApp {
    pub fn new() -> GradioApp {
        let player = AudioPlayer::new();
        let library = Library::new();

        let (sender, receiver) = channel();

        // load custom stylesheet
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("style.css").as_bytes());
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &provider, 600);

        let builder = gtk::Builder::new_from_string(include_str!("window.ui"));
        let gtk_app = gtk::Application::new("de.haeckerfelix.Gradio", gio::ApplicationFlags::empty()).expect("Failed to initialize GtkApplication");
        let window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
        let page_stack: gtk::Stack = builder.get_object("page_stack").unwrap();

        let library_page: LibraryPage = Page::new(sender.clone());
        library_page.update_stations(&library.stations);
        page_stack.add_titled(library_page.container(), &library_page.name(), &library_page.title());

        let search_page: SearchPage = Page::new(sender.clone());
        page_stack.add_titled(search_page.container(), &search_page.name(), &search_page.title());

        let playerbar: gtk::ActionBar = builder.get_object("playerbar").unwrap();
        playerbar.set_visible(false);
        let station_title: gtk::Label = builder.get_object("station_title").unwrap();
        let station_subtitle: gtk::Label = builder.get_object("station_subtitle").unwrap();

        GradioApp {
            player,
            library,
            receiver,
            sender,
            builder,
            gtk_app,
            window,
            page_stack,
            library_page,
            search_page,
            playerbar,
            station_title,
            station_subtitle,
        }
    }

    pub fn run(self) {
        self.connect_signals();

        let receiver = self.receiver;
        let player = self.player;
        let playerbar = self.playerbar;
        let station_title = self.station_title;
        let station_subtitle = self.station_subtitle;
        gtk::timeout_add(50, move || {
            match receiver.try_recv() {
                Ok(Action::PlaybackStart) => player.set_playback(true),
                Ok(Action::PlaybackStop) => player.set_playback(false),
                Ok(Action::PlaybackSetStation(station)) => {
                    playerbar.set_visible(true);
                    station_title.set_text(&station.name);
                    player.set_station(&station)
                },
                Ok(Action::LibraryAdd(station)) => info!("setplayback"),
                Ok(Action::LibraryRemove(station)) => info!("setplayback"),
                Err(_) => (),
            }
            Continue(true)
        });

        self.gtk_app.run(&[]);
    }

    fn connect_signals(&self) {
        // GTK Application activate
        let window_clone = self.window.clone();
        self.gtk_app.connect_activate(move |app| {
            app.add_window(&window_clone);
            debug!("gtk application activate");
        });
    }
}
