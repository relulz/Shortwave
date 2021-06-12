// Shortwave - app.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gio::subclass::prelude::ApplicationImpl;
use glib::clone;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use crate::api::SwStation;
use crate::audio::{GCastDevice, PlaybackState, Player, Song};
use crate::config;
use crate::database::SwLibrary;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key, SettingsWindow};
use crate::ui::{about_dialog, Notification, SwApplicationWindow, SwView};

#[derive(Debug, Clone)]
pub enum Action {
    /* User Interface */
    ViewGoBack,
    ViewSet(SwView),
    ViewSetMiniPlayer(bool),
    ViewRaise,
    ViewShowNotification(Rc<Notification>),

    /* Audio Playback */
    PlaybackConnectGCastDevice(GCastDevice),
    PlaybackDisconnectGCastDevice,
    PlaybackSetStation(Box<SwStation>),
    PlaybackSet(bool),
    PlaybackToggle,
    PlaybackSetVolume(f64),
    PlaybackSaveSong(Song),

    /* Library */
    LibraryAddStations(Vec<SwStation>),
    LibraryRemoveStations(Vec<SwStation>),

    SettingsKeyChanged(Key),
}

mod imp {
    use super::*;

    pub struct SwApplication {
        pub sender: Sender<Action>,
        pub receiver: RefCell<Option<Receiver<Action>>>,

        pub window: RefCell<Option<SwApplicationWindow>>,
        pub player: Rc<Player>,
        pub library: SwLibrary,

        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwApplication {
        const NAME: &'static str = "SwApplication";
        type ParentType = gtk::Application;
        type Type = super::SwApplication;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            let window = RefCell::new(None);
            let player = Player::new(sender.clone());
            let library = SwLibrary::new(sender.clone());

            let settings = settings_manager::settings();

            Self {
                sender,
                receiver,
                window,
                player,
                library,
                settings,
            }
        }
    }

    // Implement GLib.OBject for SwApplication
    impl ObjectImpl for SwApplication {}

    // Implement Gtk.Application for SwApplication
    impl GtkApplicationImpl for SwApplication {}

    // Implement Gio.Application for SwApplication
    impl ApplicationImpl for SwApplication {
        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);
            app.set_resource_base_path(Some("/de/haeckerfelix/Shortwave/"));
        }

        fn activate(&self, app: &Self::Type) {
            debug!("gio::Application -> activate()");
            let app = app.downcast_ref::<super::SwApplication>().unwrap();

            // If the window already exists,
            // present it instead creating a new one again.
            if let Some(ref window) = *self.window.borrow() {
                window.present();
                info!("Application window presented.");
                return;
            }

            // No window available -> we have to create one
            let window = app.create_window();
            self.window.replace(Some(window));
            info!("Created application window.");

            // Setup app level GActions
            app.setup_gactions();

            // Setup action channel
            let receiver = self.receiver.borrow_mut().take().unwrap();
            receiver.attach(None, clone!(@strong app => move |action| app.process_action(action)));

            // Setup settings signal (we get notified when a key gets changed)
            self.settings.connect_changed(
                None,
                clone!(@strong self.sender as sender => move |_, key_str| {
                    let key: Key = Key::from_str(key_str).unwrap();
                    send!(sender, Action::SettingsKeyChanged(key));
                }),
            );

            // List all setting keys
            settings_manager::list_keys();

            // Small workaround to update every view to the correct sorting/order.
            send!(self.sender, Action::SettingsKeyChanged(Key::ViewSorting));
        }
    }
}

// Wrap SwApplication into a usable gtk-rs object
glib::wrapper! {
    pub struct SwApplication(ObjectSubclass<imp::SwApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

// SwApplication implementation itself
impl SwApplication {
    pub fn run() {
        info!("{} ({}) ({})", config::NAME, config::APP_ID, config::VCS_TAG);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Isahc version: {}", isahc::version());

        // Create new GObject and downcast it into SwApplication
        let app = glib::Object::new::<SwApplication>(&[("application-id", &Some(config::APP_ID)), ("flags", &gio::ApplicationFlags::empty())]).unwrap();

        // Start running gtk::Application
        app.run();
    }

    fn create_window(&self) -> SwApplicationWindow {
        let imp = imp::SwApplication::from_instance(self);
        let window = SwApplicationWindow::new(imp.sender.clone(), self.clone(), imp.player.clone());

        // Load custom styling
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/Shortwave/gtk/style.css");
        gtk::StyleContext::add_provider_for_display(&gdk::Display::default().unwrap(), &p, 500);

        // Set initial view
        window.set_view(SwView::Library);

        window.present();
        window
    }

    fn setup_gactions(&self) {
        let window = self.active_window().unwrap();

        // app.show-preferences
        action!(
            self,
            "show-preferences",
            clone!(@weak window => move |_, _| {
                let settings_window = SettingsWindow::new(&window.upcast());
                settings_window.show();
            })
        );
        self.set_accels_for_action("app.show-preferences", &["<primary>comma"]);

        // app.quit
        action!(
            self,
            "quit",
            clone!(@weak self as this => move |_, _| {
                this.quit();
            })
        );
        self.set_accels_for_action("app.quit", &["<primary>q"]);

        // app.about
        action!(
            self,
            "about",
            clone!(@weak window => move |_, _| {
                about_dialog::show_about_dialog(&window);
            })
        );
    }

    pub fn library(&self) -> SwLibrary {
        let imp = imp::SwApplication::from_instance(self);
        imp.library.clone()
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        let imp = imp::SwApplication::from_instance(self);

        match action {
            Action::ViewGoBack => imp.window.borrow().as_ref().unwrap().go_back(),
            Action::ViewSet(view) => imp.window.borrow().as_ref().unwrap().set_view(view),
            Action::ViewRaise => imp.window.borrow().as_ref().unwrap().present_with_time((glib::monotonic_time() / 1000) as u32),
            Action::ViewSetMiniPlayer(enable) => imp.window.borrow().as_ref().unwrap().enable_mini_player(enable),
            Action::ViewShowNotification(notification) => imp.window.borrow().as_ref().unwrap().show_notification(notification),
            Action::PlaybackConnectGCastDevice(device) => imp.player.connect_to_gcast_device(device),
            Action::PlaybackDisconnectGCastDevice => imp.player.disconnect_from_gcast_device(),
            Action::PlaybackSetStation(station) => {
                imp.player.set_station(*station);
                imp.window.borrow().as_ref().unwrap().show_player_widget();
            }
            Action::PlaybackSet(true) => imp.player.set_playback(PlaybackState::Playing),
            Action::PlaybackSet(false) => imp.player.set_playback(PlaybackState::Stopped),
            Action::PlaybackToggle => imp.player.toggle_playback(),
            Action::PlaybackSetVolume(volume) => imp.player.set_volume(volume),
            Action::PlaybackSaveSong(song) => imp.player.save_song(song),
            Action::LibraryAddStations(stations) => imp.library.add_stations(stations),
            Action::LibraryRemoveStations(stations) => imp.library.remove_stations(stations),
            Action::SettingsKeyChanged(key) => self.apply_settings_changes(key),
        }
        glib::Continue(true)
    }

    fn apply_settings_changes(&self, key: Key) {
        let imp = imp::SwApplication::from_instance(self);

        debug!("Settings key changed: {:?}", &key);
        match key {
            Key::ViewSorting | Key::ViewOrder => {
                let sorting: SwSorting = SwSorting::from_str(&settings_manager::string(Key::ViewSorting)).unwrap();
                let order = settings_manager::string(Key::ViewOrder);
                let descending = order == "Descending";
                imp.window.borrow().as_ref().unwrap().set_sorting(sorting, descending);
            }
            _ => (),
        }
    }
}
