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
use glib::{GEnum, ParamSpec, ToValue};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{Action, SwApplication};
use crate::audio::Player;
use crate::config;
use crate::model::SwSorting;
use crate::settings::{settings_manager, Key};
use crate::ui::pages::*;
use crate::ui::Notification;

#[derive(Display, Copy, Debug, Clone, EnumString, PartialEq, GEnum)]
#[repr(u32)]
#[genum(type_name = "SwSwView")]
pub enum SwView {
    Library,
    Discover,
    Search,
    Player,
}

impl Default for SwView {
    fn default() -> Self {
        SwView::Library
    }
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
        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub appmenu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub default_menu: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub library_menu: TemplateChild<gio::MenuModel>,

        pub current_notification: RefCell<Option<Rc<Notification>>>,
        pub view: RefCell<SwView>,
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

    impl ObjectImpl for SwApplicationWindow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpec::new_enum(
                    "view",
                    "View",
                    "View",
                    SwView::static_type(),
                    SwView::default() as i32,
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "view" => self.view.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "view" => {
                    *self.view.borrow_mut() = value.get().unwrap();
                    obj.update_view();
                }
                _ => unimplemented!(),
            }
        }
    }

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
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

// SwApplicationWindow implementation itself
impl SwApplicationWindow {
    pub fn new(sender: Sender<Action>, app: SwApplication, player: Rc<Player>) -> Self {
        // Create new GObject and downcast it into SwApplicationWindow
        let window = glib::Object::new::<Self>(&[]).unwrap();
        app.add_window(&window);

        window.setup_widgets(sender.clone(), player);
        window.setup_signals(sender.clone());
        window.setup_gactions(sender);

        // Library is the default page
        window.set_view(SwView::Library);

        window
    }

    pub fn setup_widgets(&self, sender: Sender<Action>, player: Rc<Player>) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        // Init pages
        imp.library_page.init(sender.clone());
        imp.discover_page.init(sender.clone());
        imp.search_page.init(sender);

        // Wire everything up
        imp.mini_controller_box.append(&player.mini_controller_widget);
        imp.toolbar_controller_box.append(&player.toolbar_controller_widget);
        imp.window_flap.set_flap(Some(&player.widget));

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            let ctx = self.style_context();
            ctx.add_class("devel");
        }

        // Restore window geometry
        let width = settings_manager::integer(Key::WindowWidth);
        let height = settings_manager::integer(Key::WindowHeight);
        self.set_default_size(width, height);
    }

    fn setup_signals(&self, _sender: Sender<Action>) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        // dark mode
        let s = settings_manager::settings();
        let gtk_s = gtk::Settings::default().unwrap();
        s.bind("dark-mode", &gtk_s, "gtk-application-prefer-dark-theme").flags(gio::SettingsBindFlags::GET).build();

        // flap
        imp.window_flap.get().connect_folded_notify(clone!(@strong self as this => move |_| {
            this.update_visible_view();
        }));
        imp.window_flap.get().connect_reveal_flap_notify(clone!(@strong self as this => move |_| {
            this.update_visible_view();
        }));

        // search_button
        imp.search_button.connect_toggled(clone!(@strong self as this => move |search_button| {
            let imp = imp::SwApplicationWindow::from_instance(&this);
            if search_button.is_active(){
                this.set_view(SwView::Search);
            }else if *imp.view.borrow() != SwView::Player {
                this.set_view(SwView::Discover);
            }
        }));

        // window gets closed
        self.connect_close_request(move |window| {
            debug!("Saving window geometry.");
            let width = window.default_size().0;
            let height = window.default_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            glib::signal::Inhibit(false)
        });
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        let app = self.application().unwrap();

        // win.show-help-overlay
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/help_overlay.ui");
        get_widget!(builder, gtk::ShortcutsWindow, help_overlay);
        self.set_help_overlay(Some(&help_overlay));
        app.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);

        // win.open-radio-browser-info
        action!(self, "open-radio-browser-info", |_, _| {
            open::that("https://www.radio-browser.info/").expect("Could not open webpage.");
        });

        // win.create-new-station
        action!(self, "create-new-station", |_, _| {
            open::that("https://www.radio-browser.info/#!/add").expect("Could not open webpage.");
        });

        // win.go-back
        action!(
            self,
            "go-back",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewGoBack);
            })
        );
        app.set_accels_for_action("win.go-back", &["Escape"]);

        // win.show-discover
        action!(
            self,
            "show-discover",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewSet(SwView::Discover));
            })
        );
        app.set_accels_for_action("win.show-discover", &["<primary>d"]);

        // win.show-search
        action!(
            self,
            "show-search",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewSet(SwView::Search));
            })
        );
        app.set_accels_for_action("win.show-search", &["<primary>f"]);

        // win.show-library
        action!(
            self,
            "show-library",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewSet(SwView::Library));
            })
        );
        app.set_accels_for_action("win.show-library", &["<primary>l"]);

        // win.show-appmenu
        action!(
            self,
            "show-appmenu",
            clone!(@strong imp.appmenu_button as appmenu_button => move |_, _| {
                appmenu_button.popup();
            })
        );
        app.set_accels_for_action("win.show-appmenu", &["F10"]);

        // win.toggle-playback
        action!(
            self,
            "toggle-playback",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::PlaybackToggle);
            })
        );
        app.set_accels_for_action("win.toggle-playback", &["<primary>space"]);

        // win.disable-mini-player
        action!(
            self,
            "disable-mini-player",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewSetMiniPlayer(false));
            })
        );

        // win.enable-mini-player
        action!(
            self,
            "enable-mini-player",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::ViewSetMiniPlayer(true));
            })
        );

        // Sort / Order menu
        let sorting_action = settings_manager::create_action(Key::ViewSorting);
        self.add_action(&sorting_action);

        let order_action = settings_manager::create_action(Key::ViewOrder);
        self.add_action(&order_action);
    }

    pub fn show_player_widget(&self) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        imp.toolbar_controller_revealer.set_visible(true);
        imp.window_flap.set_locked(false);

        self.update_visible_view();
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

    pub fn set_sorting(&self, sorting: SwSorting, descending: bool) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        imp.library_page.get().set_sorting(sorting, descending);
    }

    pub fn set_view(&self, view: SwView) {
        self.set_property("view", &view).unwrap()
    }

    pub fn enable_mini_player(&self, enable: bool) {
        debug!("Enable mini player: {:?}", enable);

        // GTK sometimes refuses to set the proper size, so we have to apply a workaround here...
        // For some reasons it works reliable this way.
        let duration = std::time::Duration::from_millis(100);
        self.set_default_size(0, 0);

        if enable {
            glib::timeout_add_local(
                duration,
                clone!(@weak self as this => @default-return glib::Continue(false), move||{
                    this.unmaximize();
                    this.set_default_size(450, 105);
                    glib::Continue(false)
                }),
            );
        } else {
            glib::timeout_add_local(
                duration,
                clone!(@weak self as this => @default-return glib::Continue(false), move||{
                    this.set_default_size(950, 650);
                    glib::Continue(false)
                }),
            );
        }
    }

    pub fn go_back(&self) {
        debug!("Go back to previous view");
        let imp = imp::SwApplicationWindow::from_instance(self);

        if *imp.view.borrow() == SwView::Player {
            imp.window_flap.set_reveal_flap(false);
        } else {
            imp.window_leaflet.navigate(adw::NavigationDirection::Back);
        }

        self.update_visible_view();
    }

    fn update_visible_view(&self) {
        let imp = imp::SwApplicationWindow::from_instance(self);

        let view = if imp.window_flap.is_folded() && imp.window_flap.reveals_flap() {
            SwView::Player
        } else {
            let leaflet_child = imp.window_leaflet.visible_child().unwrap();
            if leaflet_child == imp.library_page.get() {
                SwView::Library
            } else if leaflet_child == imp.discover_page.get() {
                SwView::Discover
            } else if leaflet_child == imp.search_page.get() {
                SwView::Search
            } else {
                panic!("Unknown leaflet child")
            }
        };

        debug!("Update visible view to {:?}", view);
        self.set_view(view);
    }

    fn update_view(&self) {
        let imp = imp::SwApplicationWindow::from_instance(self);
        let view = *imp.view.borrow();
        debug!("Set view to {:?}", view);

        // Not enough place to display player sidebar and content side by side (eg. mobile phones)
        let slim_mode = imp.window_flap.is_folded();
        // Wether the player widgets (sidebar / bottom toolbar) should get display or not.
        let player_activated = !imp.window_flap.is_locked();

        if player_activated {
            if slim_mode && view == SwView::Player {
                imp.window_flap.set_reveal_flap(true);
                imp.toolbar_controller_revealer.set_reveal_child(false);
            } else if slim_mode {
                imp.window_flap.set_reveal_flap(false);
                imp.toolbar_controller_revealer.set_reveal_child(true);
            } else {
                imp.window_flap.set_reveal_flap(true);
                imp.toolbar_controller_revealer.set_reveal_child(false);
            }
        }

        // Show requested view / page
        match view {
            SwView::Library => {
                imp.window_leaflet.set_visible_child(&imp.library_page.get());
                imp.appmenu_button.set_menu_model(Some(&imp.library_menu.get()));
                imp.search_revealer.set_reveal_child(false);
                imp.add_button.set_visible(true);
                imp.back_button.set_visible(false);
            }
            SwView::Discover => {
                imp.window_leaflet.set_visible_child(&imp.discover_page.get());
                imp.appmenu_button.set_menu_model(Some(&imp.default_menu.get()));
                imp.search_button.set_active(false);
                imp.search_revealer.set_reveal_child(true);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
            SwView::Search => {
                imp.window_leaflet.set_visible_child(&imp.search_page.get());
                imp.appmenu_button.set_menu_model(Some(&imp.default_menu.get()));
                imp.search_button.set_active(true);
                imp.search_revealer.set_reveal_child(true);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
            SwView::Player => {
                imp.window_flap.set_reveal_flap(true);
                imp.search_button.set_active(false);
                imp.add_button.set_visible(false);
                imp.back_button.set_visible(true);
            }
        }
    }
}
