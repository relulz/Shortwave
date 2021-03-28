// Shortwave - window.rs
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

use adw::subclass::prelude::*;
use glib::clone;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{gio, glib};

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{Action, SwApplication, SwApplicationPrivate};
use crate::config;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key, SettingsWindow};
use crate::ui::pages::*;
use crate::ui::{about_dialog, Notification};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Library,
    Discover,
    Search,
    Player,
}

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/window.ui")]
    pub struct SwApplicationWindow {
        #[template_child]
        pub library_page: TemplateChild<SwLibraryPage>,
        #[template_child]
        pub discover_page: TemplateChild<SwDiscoverPage>,
        #[template_child]
        pub search_page: TemplateChild<SwSearchPage>,

        #[template_child]
        pub mini_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub toolbar_controller_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub window_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub window_flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,

        pub current_notification: RefCell<Option<Rc<Notification>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwApplicationWindow {
        const NAME: &'static str = "SwApplicationWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::SwApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Implement GLib.Object for SwApplicationWindow
    impl ObjectImpl for SwApplicationWindow {}

    // Implement Gtk.Widget for SwApplicationWindow
    impl WidgetImpl for SwApplicationWindow {}

    // Implement Gtk.Window for SwApplicationWindow
    impl WindowImpl for SwApplicationWindow {}

    // Implement Gtk.ApplicationWindow for SwApplicationWindow
    impl ApplicationWindowImpl for SwApplicationWindow {}

    // Implement Adw.ApplicationWindow for SwApplicationWindow
    impl AdwApplicationWindowImpl for SwApplicationWindow {}
}

// Wrap imp::SwApplicationWindow into a usable gtk-rs object
glib::wrapper! {
    pub struct SwApplicationWindow(
        ObjectSubclass<imp::SwApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

// SwApplicationWindow implementation itself
impl SwApplicationWindow {
    pub fn new(sender: Sender<Action>, app: SwApplication) -> Self {
        // Create new GObject and downcast it into SwApplicationWindow
        let window = glib::Object::new::<Self>(&[]).unwrap();
        app.add_window(&window.clone());

        window.setup_widgets(sender.clone());
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);
        window
    }

    pub fn setup_widgets(&self, sender: Sender<Action>) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        let app: SwApplication = self.get_application().unwrap().downcast::<SwApplication>().unwrap();
        let app_private = SwApplicationPrivate::from_instance(&app);

        // Init pages
        imp.library_page.init(sender.clone());
        imp.discover_page.init(sender.clone());
        imp.search_page.init(sender.clone());

        // Wire everything up
        imp.mini_controller_box.append(&app_private.player.mini_controller_widget);
        imp.toolbar_controller_box.append(&app_private.player.toolbar_controller_widget);
        imp.window_flap.set_flap(Some(&app_private.player.widget));

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            let ctx = self.get_style_context();
            ctx.add_class("devel");
        }

        // Restore window geometry
        let width = settings_manager::get_integer(Key::WindowWidth);
        let height = settings_manager::get_integer(Key::WindowHeight);
        self.set_default_size(width, height);
    }

    fn setup_signals(&self, _sender: Sender<Action>) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        // dark mode
        let s = settings_manager::get_settings();
        let gtk_s = gtk::Settings::get_default().unwrap();
        s.bind("dark-mode", &gtk_s, "gtk-application-prefer-dark-theme").flags(gio::SettingsBindFlags::GET).build();

        // flap
        imp.window_flap.connect_property_folded_notify(clone!(@strong self as this => move |_| {
           this.sync_ui_state();
        }));

        // window gets closed
        self.connect_close_request(move |window| {
            debug!("Saving window geometry.");
            let width = window.get_default_size().0;
            let height = window.get_default_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            glib::signal::Inhibit(false)
        });
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from SwApplicationWindow to adw::ApplicationWindow, because SwApplicationWindow
        // currently doesn't implement GLib.ActionMap, since it's not supported in gtk-rs for subclassing (13-01-2020)
        let window = self.clone().upcast::<gtk::ApplicationWindow>();
        let app = window.get_application().unwrap();

        // win.open-radio-browser-info
        action!(window, "open-radio-browser-info", |_, _| {
            open::that("http://www.radio-browser.info/").expect("Could not open webpage.");
        });

        // win.create-new-station
        action!(window, "create-new-station", |_, _| {
            open::that("http://www.radio-browser.info/gui/#!/add").expect("Could not open webpage.");
        });

        // win.quit
        action!(
            window,
            "quit",
            clone!(@weak app => move |_, _| {
                app.quit();
            })
        );
        app.set_accels_for_action("win.quit", &["<primary>q"]);

        // win.about
        action!(
            window,
            "about",
            clone!(@weak window => move |_, _| {
                about_dialog::show_about_dialog(window);
            })
        );

        // win.show-preferences
        action!(
            window,
            "show-preferences",
            clone!(@weak window => move |_, _| {
                let settings_window = SettingsWindow::new(&window);
                settings_window.show();
            })
        );

        // win.go-back
        action!(
            window,
            "go-back",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewGoBack);
            })
        );
        app.set_accels_for_action("win.go-back", &["Escape"]);

        // win.show-discover
        action!(
            window,
            "show-discover",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewShowDiscover);
            })
        );

        // win.show-search
        action!(
            window,
            "show-search",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewShowSearch);
            })
        );
        app.set_accels_for_action("win.show-search", &["<primary>f"]);

        // win.show-library
        action!(
            window,
            "show-library",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewShowLibrary);
            })
        );

        // win.toggle-start-stop
        action!(
            window,
            "toggle-start-stop",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::PlaybackToggleStartStop);
            })
        );
        app.set_accels_for_action("win.toggle-start-stop", &["<primary>space"]);

        // win.disable-mini-player
        action!(
            window,
            "disable-mini-player",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewDisableMiniPlayer);
            })
        );

        // win.enable-mini-player
        action!(
            window,
            "enable-mini-player",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewEnableMiniPlayer);
            })
        );

        // Sort / Order menu
        let sorting_action = settings_manager::create_action(Key::ViewSorting);
        window.add_action(&sorting_action);

        let order_action = settings_manager::create_action(Key::ViewOrder);
        window.add_action(&order_action);
    }

    pub fn show_player_widget(&self) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        imp.toolbar_controller_revealer.set_visible(true);

        // Unlock player sidebar flap
        imp.window_flap.set_locked(false);

        self.sync_ui_state();
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        // Remove previous notification
        if let Some(notification) = imp.current_notification.borrow_mut().take() {
            notification.hide();
        }

        notification.show(&imp.overlay);
        *imp.current_notification.borrow_mut() = Some(notification);
    }

    pub fn set_view(&self, view: View) {
        self.update_view(view);
        self.sync_ui_state();
    }

    pub fn enable_mini_player(&self, enable: bool) {
        if enable {
            self.unmaximize();
            self.set_default_size(450, 105);
        } else {
            self.set_default_size(700, 500);
        }
    }

    pub fn set_sorting(&self, sorting: SwSorting, descending: bool) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        imp.library_page.get().set_sorting(sorting, descending);
    }

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let imp = imp::SwApplicationWindow::from_instance(self);

        // Check if current view = player sidebar
        if imp.window_flap.get_folded() && imp.window_flap.get_reveal_flap() {
            imp.window_flap.set_reveal_flap(false);
        } else {
            imp.window_leaflet.navigate(adw::NavigationDirection::Back);
        }

        imp.window_leaflet.navigate(adw::NavigationDirection::Back);

        // Make sure that the rest of the UI is correctly synced
        self.sync_ui_state();
    }

    fn sync_ui_state(&self) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        let app: SwApplication = self.get_application().unwrap().downcast().unwrap();
        let app_priv = SwApplicationPrivate::from_instance(&app);

        let leaflet_child = imp.window_leaflet.get_visible_child().unwrap();

        // Check in which state the sidebar flap is,
        // and set the corresponding view (Library|Storefront|Player)
        let current_view = if imp.window_flap.get_folded() && imp.window_flap.get_reveal_flap() {
            View::Player
        } else {
            if leaflet_child == imp.library_page.get() {
                View::Library
            } else if leaflet_child == imp.discover_page.get() {
                View::Discover
            } else {
                View::Search
            }
        };

        // Show bottom player controller toolbar when
        // sidebar flap is folded, player widget is not revealed,
        // and there is a selected station.
        let show_toolbar_controller = imp.window_flap.get_folded() && !imp.window_flap.get_reveal_flap() && app_priv.player.has_station();
        imp.toolbar_controller_revealer.set_reveal_child(show_toolbar_controller);

        // Ensure that player sidebar gets revealed
        if !show_toolbar_controller && !imp.window_flap.get_locked() {
            imp.window_flap.set_reveal_flap(true);
        }

        debug!("Setting current view as {:?}", &current_view);
        self.update_view(current_view);
    }

    fn update_view(&self, view: View) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        debug!("Set view to {:?}", &view);

        // Don't reveal sidebar flap by default
        if !imp.window_flap.get_locked() && imp.window_flap.get_folded() {
            imp.window_flap.set_reveal_flap(false);
        }

        // Show requested view / page
        match view {
            View::Library => {
                imp.window_leaflet.set_visible_child(&imp.library_page.get());
                imp.back_button.set_visible(false);
                imp.add_button.set_visible(true);
            }
            View::Discover => {
                imp.window_leaflet.set_visible_child(&imp.discover_page.get());
                imp.back_button.set_visible(true);
                imp.add_button.set_visible(false);
            }
            View::Search => {
                imp.window_leaflet.set_visible_child(&imp.search_page.get());
                imp.back_button.set_visible(true);
                imp.add_button.set_visible(false);
            }
            View::Player => {
                imp.window_flap.set_reveal_flap(true);
                imp.back_button.set_visible(true);
                imp.add_button.set_visible(false);
            }
        }
    }
}
