use gio::prelude::*;
use glib::futures::FutureExt;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use libhandy::{ViewSwitcherBarExt, ViewSwitcherExt};
use url::Url;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use crate::api::{Client, Station, StationRequest};
use crate::audio::{PlaybackState, Player};
use crate::config;
use crate::database::gradio_db;
use crate::database::Library;
use crate::discover::StoreFront;
use crate::ui::{View, Window};
use crate::utils::{Order, Sorting};

#[derive(Debug, Clone)]
pub enum Action {
    ViewShowDiscover,
    ViewShowLibrary,
    ViewShowNotification(String),
    ViewRaise,
    ViewSetSorting(Sorting, Order),
    PlaybackSetStation(Station),
    PlaybackStart,
    PlaybackStop,
    LibraryGradioImport,
    LibraryAddStations(Vec<Station>),
    LibraryRemoveStations(Vec<Station>),
    SearchFor(StationRequest), // TODO: is this neccessary?
}

pub struct App {
    gtk_app: gtk::Application,

    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    window: Window,
    player: Player,
    library: Library,
    storefront: StoreFront,
}

impl App {
    pub fn new() -> Rc<Self> {
        // Set custom style
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/Shortwave/gtk/style.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &p, 500);

        let gtk_app = gtk::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE).unwrap();
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let window = Window::new(sender.clone());
        let player = Player::new(sender.clone());
        let library = Library::new(sender.clone());
        let storefront = StoreFront::new(sender.clone());

        window.sidebar_player_box.add(&player.widget);
        window.library_box.add(&library.widget);
        window.discover_box.add(&storefront.widget);
        window.set_view(View::Library);

        window.discover_header_switcher.set_stack(Some(&storefront.discover_stack));
        window.discover_bottom_switcher.set_stack(Some(&storefront.discover_stack));

        // Help overlay
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/shortcuts.ui");
        let dialog: gtk::ShortcutsWindow = builder.get_object("shortcuts").unwrap();
        window.widget.set_help_overlay(Some(&dialog));

        let app = Rc::new(Self {
            gtk_app,
            sender,
            receiver,
            window,
            player,
            library,
            storefront,
        });

        glib::set_application_name(&config::NAME);
        glib::set_prgname(Some("shortwave"));
        gtk::Window::set_default_icon_name(config::APP_ID);

        app.setup_gaction();
        app.setup_signals();
        app
    }

    pub fn run(&self, app: Rc<Self>) {
        info!("{} ({}) ({})", config::NAME, config::APP_ID, config::VCS_TAG);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        let a = app.clone();
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| a.process_action(action));

        let args: Vec<String> = env::args().collect();
        self.gtk_app.run(&args);
        self.player.shutdown();
    }

    fn setup_gaction(&self) {
        // Quit
        let gtk_app = self.gtk_app.clone();
        self.add_gaction("quit", move |_, _| gtk_app.quit());
        self.gtk_app.set_accels_for_action("app.quit", &["<primary>q"]);

        // About
        let window = self.window.widget.clone();
        self.add_gaction("about", move |_, _| {
            Self::show_about_dialog(window.clone());
        });

        // Search / Discover / add stations
        let sender = self.sender.clone();
        self.add_gaction("discover", move |_, _| {
            sender.send(Action::ViewShowDiscover).unwrap();
        });
        self.gtk_app.set_accels_for_action("app.discover", &["<primary>f"]);

        // Import library
        let sender = self.sender.clone();
        self.add_gaction("import-gradio-library", move |_, _| {
            sender.send(Action::LibraryGradioImport).unwrap();
        });

        // Sort / Order menu
        let sort_variant = "name".to_variant();
        let sorting_action = gio::SimpleAction::new_stateful("sorting", Some(sort_variant.type_()), &sort_variant);
        self.gtk_app.add_action(&sorting_action);

        let order_variant = "ascending".to_variant();
        let order_action = gio::SimpleAction::new_stateful("order", Some(order_variant.type_()), &order_variant);
        self.gtk_app.add_action(&order_action);

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        let sender = self.sender.clone();
        sorting_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa, &sender);
        });

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        let sender = self.sender.clone();
        order_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa, &sender);
        });
    }

    fn sort_action(sorting_action: &gio::SimpleAction, order_action: &gio::SimpleAction, sender: &Sender<Action>) {
        let order_str: String = order_action.get_state().unwrap().get_str().unwrap().to_string();
        let order = match order_str.as_ref() {
            "ascending" => Order::Ascending,
            _ => Order::Descending,
        };

        let sorting_str: String = sorting_action.get_state().unwrap().get_str().unwrap().to_string();
        let sorting = match sorting_str.as_ref() {
            "language" => Sorting::Language,
            "country" => Sorting::Country,
            "state" => Sorting::State,
            "codec" => Sorting::Codec,
            "votes" => Sorting::Votes,
            "bitrate" => Sorting::Bitrate,
            _ => Sorting::Name,
        };

        sender.send(Action::ViewSetSorting(sorting, order)).unwrap();
    }

    fn add_gaction<F>(&self, name: &str, action: F)
    where
        for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&'s glib::Variant>) + 'static,
    {
        let simple_action = gio::SimpleAction::new(name, None);
        simple_action.connect_activate(action);
        self.gtk_app.add_action(&simple_action);
    }

    fn setup_signals(&self) {
        let window = self.window.widget.clone();
        self.gtk_app.connect_activate(move |app| {
            app.add_window(&window);
            window.present();
        });
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::ViewShowDiscover => self.window.set_view(View::Discover),
            Action::ViewShowLibrary => self.window.set_view(View::Library),
            Action::ViewRaise => self.window.widget.present_with_time((glib::get_monotonic_time() / 1000) as u32),
            Action::ViewShowNotification(text) => self.window.show_notification(text),
            Action::ViewSetSorting(sorting, order) => self.library.set_sorting(sorting, order),
            Action::PlaybackSetStation(station) => self.player.set_station(station.clone()),
            Action::PlaybackStart => self.player.set_playback(PlaybackState::Playing),
            Action::PlaybackStop => self.player.set_playback(PlaybackState::Stopped),
            Action::LibraryGradioImport => self.import_gradio_library(),
            Action::LibraryAddStations(stations) => self.library.add_stations(stations),
            Action::LibraryRemoveStations(stations) => self.library.remove_stations(stations),
            Action::SearchFor(data) => self.storefront.search_for(data),
        }
        glib::Continue(true)
    }

    fn show_about_dialog(window: gtk::ApplicationWindow) {
        let vcs_tag = config::VCS_TAG;
        let version_suffix: String = match config::PROFILE {
            "development" => format!("\n(Development Commit {})", vcs_tag).to_string(),
            _ => "".to_string(),
        };

        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name(config::NAME);
        dialog.set_logo_icon_name(Some(config::APP_ID));
        dialog.set_comments(Some("A web radio client"));
        dialog.set_copyright(Some("© 2019 Felix Häcker"));
        dialog.set_license_type(gtk::License::Gpl30);
        dialog.set_version(Some(format!("{}{}", config::VERSION, version_suffix).as_str()));
        dialog.set_transient_for(Some(&window));
        dialog.set_modal(true);

        dialog.set_authors(&["Felix Häcker"]);
        dialog.set_artists(&["Tobias Bernard"]);

        dialog.connect_response(|dialog, _| dialog.destroy());
        dialog.show();
    }

    fn import_gradio_library(&self) {
        let import_dialog = gtk::FileChooserNative::new(
            Some("Select database to import"),
            Some(&self.window.widget),
            gtk::FileChooserAction::Open,
            Some("Import"),
            Some("Cancel"),
        );

        // Set filechooser filters
        let filter = gtk::FileFilter::new();
        import_dialog.set_filter(&filter);
        filter.add_mime_type("application/x-sqlite3");
        filter.add_mime_type("application/vnd.sqlite3");

        if gtk::ResponseType::from(import_dialog.run()) == gtk::ResponseType::Accept {
            let path = import_dialog.get_file().unwrap().get_path().unwrap();
            debug!("Import path: {:?}", path);

            // Get station identifiers
            let ids = gradio_db::read_database(path);
            let message = format!("Importing {} stations...", ids.len());
            self.sender.send(Action::ViewShowNotification(message)).unwrap();

            // Get actual stations from identifiers
            let client = Client::new(Url::parse("http://www.radio-browser.info/webservice/").unwrap());
            let sender = self.sender.clone();
            let fut = client.get_stations_by_identifiers(ids).map(move |stations| {
                sender.send(Action::LibraryAddStations(stations.clone())).unwrap();

                let message = format!("Imported {} stations!", stations.len());
                sender.send(Action::ViewShowNotification(message)).unwrap();
            });

            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);
        }
        import_dialog.destroy();
    }
}
