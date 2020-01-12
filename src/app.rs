use gio::subclass::prelude::ApplicationImpl;
use gio::{self, prelude::*, ApplicationFlags, SettingsExt};
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk::subclass::application::GtkApplicationImpl;
use libhandy::{ViewSwitcherBarExt, ViewSwitcherExt};

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::str::FromStr;

use crate::api::{Station, StationRequest};
use crate::audio::{GCastDevice, PlaybackState, Player, Song};
use crate::config;
use crate::database::Library;
use crate::discover::StoreFront;
use crate::settings::{settings_manager, Key};
use crate::ui::{Notification, View, Window};
use crate::utils::{Order, Sorting};

#[derive(Debug, Clone)]
pub enum Action {
    ViewShowDiscover,
    ViewShowLibrary,
    ViewShowPlayer,
    ViewRaise,
    ViewShowNotification(Rc<Notification>),
    PlaybackConnectGCastDevice(GCastDevice),
    PlaybackDisconnectGCastDevice,
    PlaybackSetStation(Station),
    PlaybackStart,
    PlaybackStop,
    PlaybackSetVolume(f64),
    PlaybackSaveSong(Song),
    LibraryAddStations(Vec<Station>),
    LibraryRemoveStations(Vec<Station>),
    SearchFor(StationRequest), // TODO: is this neccessary?,
    SettingsKeyChanged(Key),
}

pub struct SwApplicationPrivate {
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    window: RefCell<Option<Window>>,
    player: Player,
    library: Library,
    storefront: StoreFront,

    settings: gio::Settings,
}

impl ObjectSubclass for SwApplicationPrivate {
    const NAME: &'static str = "SwApplication";
    type ParentType = gtk::Application;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let window = RefCell::new(None);
        let player = Player::new(sender.clone());
        let library = Library::new(sender.clone());
        let storefront = StoreFront::new(sender.clone());

        let settings = settings_manager::get_settings();

        Self {
            sender,
            receiver,
            window,
            player,
            library,
            storefront,
            settings,
        }
    }
}

// Implement GLib.OBject for SwApplication
impl ObjectImpl for SwApplicationPrivate {
    glib_object_impl!();
}

// Implement Gtk.Application for SwApplication
impl GtkApplicationImpl for SwApplicationPrivate {}

// Implement Gio.Application for SwApplication
impl ApplicationImpl for SwApplicationPrivate {
    fn activate(&self, _app: &gio::Application) {
        debug!("gio::Application -> activate()");

        // If the window already exists,
        // present it instead creating a new one again.
        if let Some(ref window) = *self.window.borrow() {
            window.widget.present();
            info!("Application window presented.");
            return;
        }

        // No window available -> we have to create one
        let app = ObjectSubclass::get_instance(self).downcast::<SwApplication>().unwrap();
        let window = app.create_window();
        window.widget.present();
        window.setup_gactions();
        app.add_window(&window.widget);
        self.window.replace(Some(window));
        info!("Created application window.");

        // Setup action channel
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.process_action(action));

        // Setup settings signal (we get notified when a key gets changed)
        let sender = self.sender.clone();
        self.settings.connect_changed(move |_, key_str| {
            let key: Key = Key::from_str(key_str).unwrap();
            sender.send(Action::SettingsKeyChanged(key)).unwrap();
        });

        // List all setting keys
        settings_manager::list_keys();

        // Small workaround to update every view to the correct sorting/order.
        self.sender.send(Action::SettingsKeyChanged(Key::ViewSorting)).unwrap();
    }
}

// Wrap the SwApplicationPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct SwApplication(
        Object<subclass::simple::InstanceStruct<SwApplicationPrivate>,
        subclass::simple::ClassStruct<SwApplicationPrivate>,
        SwApplicationClass>)
        @extends gio::Application, gtk::Application;

    match fn {
        get_type => || SwApplicationPrivate::get_type().to_glib(),
    }
}

// SwApplication implementation itself
impl SwApplication {
    pub fn run() {
        info!("{} ({}) ({})", config::NAME, config::APP_ID, config::VCS_TAG);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);

        // Create new GObject and downcast it into SwApplication
        let app = glib::Object::new(SwApplication::static_type(), &[("application-id", &Some(config::APP_ID)), ("flags", &ApplicationFlags::empty())])
            .unwrap()
            .downcast::<SwApplication>()
            .unwrap();

        // Start running gtk::Application
        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&app, &args);
    }

    fn create_window(&self) -> Window {
        let self_ = SwApplicationPrivate::from_instance(self);
        let window = Window::new(self_.sender.clone(), self.clone());

        // Load custom styling
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/Shortwave/gtk/style.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &p, 500);

        // Add widgets of several components to the window
        window.mini_controller_box.add(&self_.player.mini_controller_widget);
        window.library_box.add(&self_.library.widget);
        window.discover_box.add(&self_.storefront.widget);

        // Wire everything up
        window.discover_header_switcher_wide.set_stack(Some(&self_.storefront.storefront_stack));
        window.discover_header_switcher_narrow.set_stack(Some(&self_.storefront.storefront_stack));
        window.discover_bottom_switcher.set_stack(Some(&self_.storefront.storefront_stack));

        // Set initial view
        window.set_view(View::Library);

        // Setup help overlay
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/shortcuts.ui");
        get_widget!(builder, gtk::ShortcutsWindow, shortcuts);
        window.widget.set_help_overlay(Some(&shortcuts));

        window
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let self_ = SwApplicationPrivate::from_instance(self);

        match action {
            Action::ViewShowDiscover => self_.window.borrow().as_ref().unwrap().set_view(View::Discover),
            Action::ViewShowLibrary => self_.window.borrow().as_ref().unwrap().set_view(View::Library),
            Action::ViewShowPlayer => self_.window.borrow().as_ref().unwrap().set_view(View::Player),
            Action::ViewRaise => self_.window.borrow().as_ref().unwrap().widget.present_with_time((glib::get_monotonic_time() / 1000) as u32),
            Action::ViewShowNotification(notification) => self_.window.borrow().as_ref().unwrap().show_notification(notification),
            Action::PlaybackConnectGCastDevice(device) => self_.player.connect_to_gcast_device(device),
            Action::PlaybackDisconnectGCastDevice => self_.player.disconnect_from_gcast_device(),
            Action::PlaybackSetStation(station) => {
                self_.player.set_station(station.clone());
                self_.player.show(self_.window.borrow().as_ref().unwrap().leaflet.clone());
            }
            Action::PlaybackStart => self_.player.set_playback(PlaybackState::Playing),
            Action::PlaybackStop => self_.player.set_playback(PlaybackState::Stopped),
            Action::PlaybackSetVolume(volume) => self_.player.set_volume(volume),
            Action::PlaybackSaveSong(song) => self_.player.save_song(song),
            Action::LibraryAddStations(stations) => self_.library.add_stations(stations),
            Action::LibraryRemoveStations(stations) => self_.library.remove_stations(stations),
            Action::SearchFor(data) => self_.storefront.search_for(data),
            Action::SettingsKeyChanged(key) => {
                debug!("Settings key changed: {:?}", &key);
                match key {
                    Key::ViewSorting | Key::ViewOrder => {
                        let sorting: Sorting = Sorting::from_str(&settings_manager::get_string(Key::ViewSorting)).unwrap();
                        let order: Order = Order::from_str(&settings_manager::get_string(Key::ViewOrder)).unwrap();
                        self_.library.set_sorting(sorting, order);
                    }
                    _ => (),
                }
            }
        }
        glib::Continue(true)
    }
}
