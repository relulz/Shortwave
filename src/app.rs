use gio::prelude::*;
use glib::futures::FutureExt;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use libhandy::{ViewSwitcherBarExt, ViewSwitcherExt};
use url::Url;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::str::FromStr;

use crate::api::{Client, Station, StationRequest};
use crate::audio::{PlaybackState, Player, Song};
use crate::config;
use crate::database::gradio_db;
use crate::database::Library;
use crate::discover::StoreFront;
use crate::ui::{View, Window, Notification};
use crate::utils::{Order, Sorting};
use crate::settings::{Key, SettingsManager, SettingsWindow};

#[derive(Debug, Clone)]
pub enum Action {
    ViewShowDiscover,
    ViewShowLibrary,
    ViewShowPlayer,
    ViewShowSettings,
    ViewShowNotification(Rc<Notification>),
    ViewRaise,
    PlaybackSetStation(Station),
    PlaybackStart,
    PlaybackStop,
    PlaybackSetVolume(f64),
    PlaybackSaveSong(Song),
    LibraryGradioImport,
    LibraryAddStations(Vec<Station>),
    LibraryRemoveStations(Vec<Station>),
    SearchFor(StationRequest), // TODO: is this neccessary?,
    SettingsKeyChanged(Key),
}

pub struct App {
    gtk_app: gtk::Application,

    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    window: Window,
    player: Player,
    library: Library,
    storefront: StoreFront,

    settings: SettingsManager,
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

        window.player_box.add(&player.widget);
        window.mini_controller_box.add(&player.mini_controller_widget);
        window.library_box.add(&library.widget);
        window.discover_box.add(&storefront.widget);
        window.set_view(View::Library);

        window.discover_header_switcher.set_stack(Some(&storefront.discover_stack));
        window.discover_bottom_switcher.set_stack(Some(&storefront.discover_stack));

        // Create new SettingsManager which notifies about settings changes
        let settings = SettingsManager::new(sender.clone());

        // Help overlay
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/shortcuts.ui");
        get_widget!(builder, gtk::ShortcutsWindow, shortcuts);
        window.widget.set_help_overlay(Some(&shortcuts));

        // Small workaround to update every view to the correct sorting/order.
        sender.send(Action::SettingsKeyChanged(Key::ViewSorting)).unwrap();

        let app = Rc::new(Self {
            gtk_app,
            sender,
            receiver,
            window,
            player,
            library,
            storefront,
            settings,
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

        self.settings.list_keys();

        let a = app.clone();
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| a.process_action(action));

        let args: Vec<String> = env::args().collect();
        self.gtk_app.run(&args);
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

        // Preferences
        let sender = self.sender.clone();
        self.add_gaction("preferences", move |_, _| {
            sender.send(Action::ViewShowSettings).unwrap();
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
        let sort_variant = SettingsManager::get_string(Key::ViewSorting).to_variant();
        let sorting_action = gio::SimpleAction::new_stateful("sorting", Some(sort_variant.type_()), &sort_variant);
        self.gtk_app.add_action(&sorting_action);

        let order_variant = SettingsManager::get_string(Key::ViewOrder).to_variant();
        let order_action = gio::SimpleAction::new_stateful("order", Some(order_variant.type_()), &order_variant);
        self.gtk_app.add_action(&order_action);

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        sorting_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa);
        });

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        order_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa);
        });
    }

    fn sort_action(sorting_action: &gio::SimpleAction, order_action: &gio::SimpleAction) {
        let sorting_str: String = sorting_action.get_state().unwrap().get_str().unwrap().to_string();
        let order_str: String = order_action.get_state().unwrap().get_str().unwrap().to_string();

        if SettingsManager::get_string(Key::ViewSorting) != sorting_str {
            SettingsManager::set_string(Key::ViewSorting, sorting_str);
        }
        if SettingsManager::get_string(Key::ViewOrder) != order_str {
            SettingsManager::set_string(Key::ViewOrder, order_str);
        }
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
            Action::ViewShowPlayer => self.window.set_view(View::Player),
            Action::ViewShowSettings => self.show_settings_window(),
            Action::ViewRaise => self.window.widget.present_with_time((glib::get_monotonic_time() / 1000) as u32),
            Action::ViewShowNotification(notification) => self.window.show_notification(notification),
            Action::PlaybackSetStation(station) => self.player.set_station(station.clone()),
            Action::PlaybackStart => self.player.set_playback(PlaybackState::Playing),
            Action::PlaybackStop => self.player.set_playback(PlaybackState::Stopped),
            Action::PlaybackSetVolume(volume) => self.player.set_volume(volume),
            Action::PlaybackSaveSong(song) => self.player.save_song(song),
            Action::LibraryGradioImport => self.import_gradio_library(),
            Action::LibraryAddStations(stations) => self.library.add_stations(stations),
            Action::LibraryRemoveStations(stations) => self.library.remove_stations(stations),
            Action::SearchFor(data) => self.storefront.search_for(data),
            Action::SettingsKeyChanged(key) => {
                match key {
                    Key::DarkMode => self.window.update_dark_mode(),
                    Key::ViewSorting | Key::ViewOrder => {
                        let sorting: Sorting = Sorting::from_str(&SettingsManager::get_string(Key::ViewSorting)).unwrap();
                        let order: Order = Order::from_str(&SettingsManager::get_string(Key::ViewOrder)).unwrap();

                        self.library.set_sorting(sorting, order);
                    },
                    _ => (),
                }
            },
        }
        glib::Continue(true)
    }

    fn show_settings_window(&self) {
        let settings_window = SettingsWindow::new(&self.window.widget);
        settings_window.show();
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
        dialog.set_comments(Some("Listen to internet radio"));
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
            let spinner_notification = Notification::new_spinner (&message);
            self.sender.send(Action::ViewShowNotification(spinner_notification.clone())).unwrap();

            // Get actual stations from identifiers
            let client = Client::new(Url::parse(&SettingsManager::get_string(Key::ApiServer)).unwrap());
            let sender = self.sender.clone();
            let fut = client.get_stations_by_identifiers(ids).map(move |stations| {
                spinner_notification.hide();
                match stations{
                    Ok(stations) => {
                        sender.send(Action::LibraryAddStations(stations.clone())).unwrap();

                        let message = format!("Imported {} stations!", stations.len());
                        let notification = Notification::new_info(&message);
                        sender.send(Action::ViewShowNotification(notification)).unwrap();
                    },
                    Err(err) => {
                        let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                        sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
                    }
                }
            });

            let ctx = glib::MainContext::default();
            ctx.spawn_local(fut);
        }
        import_dialog.destroy();
    }
}
