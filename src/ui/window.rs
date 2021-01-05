// Shortwave - window.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libhandy::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{Action, SwApplication, SwApplicationPrivate};
use crate::config;
use crate::settings::{settings_manager, Key, SettingsWindow};
use crate::ui::{about_dialog, Notification};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Storefront,
    Library,
    Player,
}

pub struct SwApplicationWindowPrivate {
    window_builder: gtk::Builder,
    //sidebar_flap: libhandy::Flap,
    sidebar_flap: gtk::Box,
    current_notification: RefCell<Option<Rc<Notification>>>,
}

impl ObjectSubclass for SwApplicationWindowPrivate {
    const NAME: &'static str = "SwApplicationWindow";
    type ParentType = libhandy::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;
    type Type = super::SwApplicationWindow;

    glib::object_subclass!();

    fn new() -> Self {
        let window_builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/window.ui");
        let current_notification = RefCell::new(None);

        // TODO: Re-add HdyFlap as soon it gets merged in libhandy
        //let sidebar_flap = libhandy::Flap::new();
        let sidebar_flap = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        Self {
            window_builder,
            sidebar_flap,
            current_notification,
        }
    }
}

// Implement GLib.OBject for SwApplicationWindow
impl ObjectImpl for SwApplicationWindowPrivate {}

// Implement Gtk.Widget for SwApplicationWindow
impl WidgetImpl for SwApplicationWindowPrivate {}

// Implement Gtk.Window for SwApplicationWindow
impl WindowImpl for SwApplicationWindowPrivate {}

// Implement Gtk.ApplicationWindow for SwApplicationWindow
impl gtk::subclass::prelude::ApplicationWindowImpl for SwApplicationWindowPrivate {}

// Implement Hdy.ApplicationWindow for SwApplicationWindow
impl libhandy::subclass::prelude::ApplicationWindowImpl for SwApplicationWindowPrivate {}

// Wrap SwApplicationWindowPrivate into a usable gtk-rs object
glib::wrapper! {
    pub struct SwApplicationWindow(
        ObjectSubclass<SwApplicationWindowPrivate>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow;
}

// SwApplicationWindow implementation itself
impl SwApplicationWindow {
    pub fn new(sender: Sender<Action>, app: SwApplication) -> Self {
        // Create new GObject and downcast it into SwApplicationWindow
        let window = glib::Object::new::<SwApplicationWindow>(&[]).unwrap();

        app.add_window(&window.clone());
        window.setup_widgets();
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);
        window
    }

    pub fn setup_widgets(&self) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        let app: SwApplication = self.get_application().unwrap().downcast::<SwApplication>().unwrap();
        let app_private = SwApplicationPrivate::from_instance(&app);

        // Add headerbar/content to the window itself
        get_widget!(self_.window_builder, gtk::Box, window);
        libhandy::ApplicationWindowExt::set_child(self.upcast_ref::<libhandy::ApplicationWindow>(), Some(&window));

        // Wire everything up
        get_widget!(self_.window_builder, gtk::Box, mini_controller_box);
        get_widget!(self_.window_builder, gtk::Box, library_page);
        get_widget!(self_.window_builder, gtk::Box, storefront_page);
        get_widget!(self_.window_builder, gtk::Box, toolbar_controller_box);
        get_widget!(self_.window_builder, libhandy::Leaflet, window_leaflet);
        get_widget!(self_.window_builder, gtk::Overlay, overlay);

        self_.sidebar_flap.append(&window_leaflet);
        //self_.sidebar_flap.set_reveal_flap(false);
        //self_.sidebar_flap.set_locked(true);
        //self_.sidebar_flap.set_flap_position(gtk::PackType::End);
        //self_.sidebar_flap.set_flap(&app_private.player.widget);
        self_.sidebar_flap.append(&app_private.player.widget);

        overlay.set_child(Some(&self_.sidebar_flap));

        mini_controller_box.append(&app_private.player.mini_controller_widget);
        library_page.append(&app_private.library.widget);
        storefront_page.append(&app_private.storefront.widget);
        toolbar_controller_box.append(&app_private.player.toolbar_controller_widget);

        // Make sure that the headerbars are correctly synced
        let headergroup = libhandy::HeaderGroup::new();
        headergroup.add_gtk_header_bar(&app_private.library.header);
        headergroup.add_gtk_header_bar(&app_private.storefront.header);
        headergroup.add_gtk_header_bar(&app_private.player.header);

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

    fn setup_signals(&self, sender: Sender<Action>) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);

        // dark mode
        let s = settings_manager::get_settings();
        let gtk_s = gtk::Settings::get_default().unwrap();
        s.bind("dark-mode", &gtk_s, "gtk-application-prefer-dark-theme", gio::SettingsBindFlags::GET);

        // flap
        //self_.sidebar_flap.connect_property_folded_notify(clone!(@strong self as this => move |_| {
        //    this.sync_ui_state();
        //}));

        // window gets closed
        self.connect_delete_event(move |window, _| {
            debug!("Saving window geometry.");
            let width = window.get_default_size().0;
            let height = window.get_default_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            glib::signal::Inhibit(false)
        });

        // back button (mouse)
        self.connect_button_press_event(clone!(@strong sender => move |_, event|{
            if event.get_button() == 8 {
                send!(sender, Action::ViewShowLibrary);
            }
            glib::signal::Inhibit(false)
        }));
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from SwApplicationWindow to libhandy::ApplicationWindow, because SwApplicationWindow
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
        app.set_accels_for_action("win.show-discover", &["<primary>f"]);

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
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, gtk::Revealer, toolbar_controller_revealer);
        toolbar_controller_revealer.set_visible(true);

        // Unlock player sidebar flap
        //self_.sidebar_flap.set_locked(false);

        self.sync_ui_state();
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, gtk::Overlay, overlay);

        // Remove previous notification
        if let Some(notification) = self_.current_notification.borrow_mut().take() {
            notification.hide();
        }

        notification.show(&overlay);
        *self_.current_notification.borrow_mut() = Some(notification);
    }

    pub fn set_view(&self, view: View) {
        self.update_view(view);
        self.sync_ui_state();
    }

    pub fn enable_mini_player(&self, enable: bool) {
        if enable {
            self.unmaximize();
            self.set_default_size(425, 125);
        } else {
            self.set_default_size(700, 500);
        }
    }

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let self_ = SwApplicationWindowPrivate::from_instance(self);

        // Check if current view = player sidebar
        //if self_.sidebar_flap.get_folded() && self_.sidebar_flap.get_reveal_flap() {
        //    self_.sidebar_flap.set_reveal_flap(false);
        //} else {
        //    get_widget!(self_.window_builder, libhandy::Leaflet, window_leaflet);
        //    window_leaflet.navigate(libhandy::NavigationDirection::Back);
        //}
        get_widget!(self_.window_builder, libhandy::Leaflet, window_leaflet);
        window_leaflet.navigate(libhandy::NavigationDirection::Back);

        // Make sure that the rest of the UI is correctly synced
        self.sync_ui_state();
    }

    fn sync_ui_state(&self) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, libhandy::Leaflet, window_leaflet);
        get_widget!(self_.window_builder, gtk::Revealer, toolbar_controller_revealer);

        let leaflet_child_name = window_leaflet.get_visible_child_name().unwrap();

        // Check in which state the sidebar flap is,
        // and set the corresponding view (Library|Storefront|Player)
        //let current_view = if self_.sidebar_flap.get_folded() && self_.sidebar_flap.get_reveal_flap() {
        //    View::Player
        //} else {
        //    if leaflet_child_name == "storefront" {
        //        View::Storefront
        //    } else {
        //        View::Library
        //    }
        //};
        let current_view = if leaflet_child_name == "storefront" { View::Storefront } else { View::Library };

        // Show bottom player controller toolbar when sidebar flap is folded and player widget is not revealed
        //let show_toolbar_controller = self_.sidebar_flap.get_folded() && !self_.sidebar_flap.get_reveal_flap();
        //toolbar_controller_revealer.set_reveal_child(show_toolbar_controller);

        // Ensure that player sidebar gets revealed
        //if !show_toolbar_controller && !self_.sidebar_flap.get_locked() {
        //    self_.sidebar_flap.set_reveal_flap(true);
        //}

        debug!("Setting current view as {:?}", &current_view);
        self.update_view(current_view);
    }

    fn update_view(&self, view: View) {
        debug!("Set view to {:?}", &view);

        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, libhandy::Leaflet, window_leaflet);

        let app: SwApplication = self.get_application().unwrap().downcast().unwrap();
        let app_priv = SwApplicationPrivate::from_instance(&app);

        // Don't reveal sidebar flap by default
        //if !self_.sidebar_flap.get_locked() && self_.sidebar_flap.get_folded() {
        //    self_.sidebar_flap.set_reveal_flap(false);
        //}

        // Show requested view / page
        match view {
            View::Storefront => {
                window_leaflet.set_visible_child_name("storefront");
                app_priv.player.set_expand_widget(false);
            }
            View::Library => {
                window_leaflet.set_visible_child_name("library");
                app_priv.player.set_expand_widget(false);
            }
            View::Player => {
                app_priv.player.set_expand_widget(true);
                //self_.sidebar_flap.set_reveal_flap(true);
            }
        }
    }
}
