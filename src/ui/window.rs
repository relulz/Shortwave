use futures_util::future::FutureExt;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::{ApplicationWindowImpl, BinImpl, ContainerImpl, WidgetImpl, WindowImpl};
use libhandy::prelude::*;
use libhandy::LeafletExt;

use std::rc::Rc;

use crate::app::{Action, SwApplication, SwApplicationPrivate};
use crate::config;
use crate::settings::{settings_manager, Key, SettingsWindow};
use crate::ui::{about_dialog, import_dialog, Notification};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Discover,
    Library,
    Player,
}

pub struct SwApplicationWindowPrivate {
    window_builder: gtk::Builder,
    menu_builder: gtk::Builder,
}

impl ObjectSubclass for SwApplicationWindowPrivate {
    const NAME: &'static str = "SwApplicationWindow";
    type ParentType = gtk::ApplicationWindow;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        let window_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/window.ui");
        let menu_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/menu/app_menu.ui");

        Self { window_builder, menu_builder }
    }
}

// Implement GLib.OBject for SwApplicationWindow
impl ObjectImpl for SwApplicationWindowPrivate {
    glib_object_impl!();
}

// Implement Gtk.Widget for SwApplicationWindow
impl WidgetImpl for SwApplicationWindowPrivate {}

// Implement Gtk.Container for SwApplicationWindow
impl ContainerImpl for SwApplicationWindowPrivate {}

// Implement Gtk.Bin for SwApplicationWindow
impl BinImpl for SwApplicationWindowPrivate {}

// Implement Gtk.Window for SwApplicationWindow
impl WindowImpl for SwApplicationWindowPrivate {}

// Implement Gtk.ApplicationWindow for SwApplicationWindow
impl ApplicationWindowImpl for SwApplicationWindowPrivate {}

// Wrap SwApplicationWindowPrivate into a usable gtk-rs object
glib_wrapper! {
    pub struct SwApplicationWindow(
        Object<subclass::simple::InstanceStruct<SwApplicationWindowPrivate>,
        subclass::simple::ClassStruct<SwApplicationWindowPrivate>,
        SwApplicationWindowClass>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow;

    match fn {
        get_type => || SwApplicationWindowPrivate::get_type().to_glib(),
    }
}

// SwApplicationWindow implementation itself
impl SwApplicationWindow {
    pub fn new(sender: Sender<Action>, app: SwApplication) -> Self {
        // Create new GObject and downcast it into SwApplicationWindow
        let window = glib::Object::new(SwApplicationWindow::static_type(), &[]).unwrap().downcast::<SwApplicationWindow>().unwrap();

        app.add_window(&window.clone());
        window.setup_widgets();
        window.setup_signals();
        window.setup_gactions(sender);
        window
    }

    pub fn setup_widgets(&self) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        let app: SwApplication = self.get_application().unwrap().downcast::<SwApplication>().unwrap();
        let app_private = SwApplicationPrivate::from_instance(&app);

        // Add headerbar/content to the window itself
        get_widget!(self_.window_builder, libhandy::HeaderBar, headerbar);
        get_widget!(self_.window_builder, gtk::Overlay, content);
        self.set_titlebar(Some(&headerbar));
        self.add(&content);

        // Wire everything up
        get_widget!(self_.window_builder, gtk::Box, mini_controller_box);
        get_widget!(self_.window_builder, gtk::Box, library_box);
        get_widget!(self_.window_builder, gtk::Box, discover_box);

        mini_controller_box.add(&app_private.player.mini_controller_widget);
        library_box.add(&app_private.library.widget);
        discover_box.add(&app_private.storefront.widget);

        get_widget!(self_.window_builder, libhandy::ViewSwitcher, discover_header_switcher_wide);
        get_widget!(self_.window_builder, libhandy::ViewSwitcher, discover_header_switcher_narrow);
        get_widget!(self_.window_builder, libhandy::ViewSwitcherBar, discover_bottom_switcher);

        discover_header_switcher_wide.set_stack(Some(&app_private.storefront.storefront_stack));
        discover_header_switcher_narrow.set_stack(Some(&app_private.storefront.storefront_stack));
        discover_bottom_switcher.set_stack(Some(&app_private.storefront.storefront_stack));

        // Set hamburger menu
        get_widget!(self_.menu_builder, gtk::PopoverMenu, popover_menu);
        get_widget!(self_.window_builder, gtk::MenuButton, appmenu_button);
        appmenu_button.set_popover(Some(&popover_menu));

        // Add devel style class for development or beta builds
        if config::PROFILE == "development" || config::PROFILE == "beta" {
            let ctx = self.get_style_context();
            ctx.add_class("devel");
        }

        // Restore window geometry
        let width = settings_manager::get_integer(Key::WindowWidth);
        let height = settings_manager::get_integer(Key::WindowHeight);
        self.resize(width, height);
    }

    fn setup_signals(&self) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);

        // dark mode
        let s = settings_manager::get_settings();
        let gtk_s = gtk::Settings::get_default().unwrap();
        s.bind("dark-mode", &gtk_s, "gtk-application-prefer-dark-theme", gio::SettingsBindFlags::GET);

        // leaflet
        get_widget!(self_.window_builder, gtk::Stack, view_stack);
        get_widget!(self_.window_builder, libhandy::Leaflet, leaflet);
        leaflet.connect_property_fold_notify(clone!(@strong self as this => move |leaflet| {
            let current_view = if leaflet.get_property_folded() && leaflet.get_visible_child_name().unwrap() == "player" {
                View::Player
            } else {
                match view_stack.get_visible_child_name().unwrap().as_str() {
                    "discover" => View::Discover,
                    _ => View::Library,
                }
            };
            this.update_view(current_view);
        }));

        // window gets closed
        self.connect_delete_event(move |window, _| {
            debug!("Saving window geometry.");
            let width = window.get_size().0;
            let height = window.get_size().1;

            settings_manager::set_integer(Key::WindowWidth, width);
            settings_manager::set_integer(Key::WindowHeight, height);
            Inhibit(false)
        });
    }

    fn setup_gactions(&self, sender: Sender<Action>) {
        // We need to upcast from SwApplicationWindow to gtk::ApplicationWindow, because SwApplicationWindow
        // currently doesn't implement GLib.ActionMap, since it's not supported in gtk-rs for subclassing (13-01-2020)
        let window = self.clone().upcast::<gtk::ApplicationWindow>();
        let app = window.get_application().unwrap();

        // win.create-new-station
        action!(window, "create-new-station", |_, _| {
            open::that("http://www.radio-browser.info/gui/#!/add").expect("Could not open webpage.");
        });

        // win.quit
        action!(
            window,
            "quit",
            clone!(@strong app => move |_, _| {
                app.quit();
            })
        );
        app.set_accels_for_action("win.quit", &["<primary>q"]);

        // win.about
        action!(
            window,
            "about",
            clone!(@strong window => move |_, _| {
                about_dialog::show_about_dialog(window.clone());
            })
        );

        // win.show-preferences
        action!(
            window,
            "show-preferences",
            clone!(@strong window => move |_, _| {
                let settings_window = SettingsWindow::new(&window);
                settings_window.show();
            })
        );

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

        // win.import-gradio-library
        action!(
            window,
            "import-gradio-library",
            clone!(@strong sender, @strong window => move |_, _| {
                let sender = sender.clone();
                let future = import_dialog::import_gradio_db(sender.clone(), window.clone()).map(move|result|{
                    match result{
                        Ok(_) => (),
                        Err(err) => {
                            let notification = Notification::new_error("Could not import library.", &err.to_string());
                            send!(sender, Action::ViewShowNotification(notification));
                        }
                    }
                });
                spawn!(future);
            })
        );

        // Sort / Order menu
        let sorting_action = settings_manager::create_action(Key::ViewSorting);
        window.add_action(&sorting_action);

        let order_action = settings_manager::create_action(Key::ViewOrder);
        window.add_action(&order_action);
    }

    pub fn show_player_widget(&self, player: gtk::Box) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, libhandy::Leaflet, leaflet);

        // We don't have to add the widget again if it's already added
        if leaflet.get_children().len() != 3 {
            let separator = gtk::Separator::new(gtk::Orientation::Vertical);
            separator.set_visible(true);
            leaflet.add(&separator);

            leaflet.add(&player);
        }
        leaflet.set_child_name(&player, Some("player"));

        // TODO: The revealer inside the player widget currently doesn't get animated.
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);
        get_widget!(self_.window_builder, gtk::Overlay, content);
        notification.show(&content);
    }

    pub fn set_view(&self, view: View) {
        self.update_view(view);
    }

    fn update_view(&self, view: View) {
        let self_ = SwApplicationWindowPrivate::from_instance(self);

        get_widget!(self_.window_builder, gtk::Revealer, bottom_switcher_revealer);
        get_widget!(self_.window_builder, gtk::Stack, bottom_switcher_stack);
        get_widget!(self_.window_builder, gtk::Stack, header_switcher_stack);
        get_widget!(self_.window_builder, gtk::Stack, view_stack);
        get_widget!(self_.window_builder, libhandy::Leaflet, leaflet);
        get_widget!(self_.menu_builder, gtk::ModelButton, sorting_mbutton);
        get_widget!(self_.menu_builder, gtk::ModelButton, library_mbutton);
        get_widget!(self_.window_builder, gtk::Button, add_button);
        get_widget!(self_.window_builder, gtk::Button, back_button);

        // Determine if window is currently in phone mode (leaflet = folded)
        let phone_mode = leaflet.get_property_folded();

        // Determine if current visible page is library
        let library_mode = view == View::Library;

        // Show or hide buttons depending on the selected view
        add_button.set_visible(library_mode);
        back_button.set_visible(!library_mode);
        sorting_mbutton.set_sensitive(library_mode);
        library_mbutton.set_sensitive(library_mode);

        // Show requested view / page
        match view {
            View::Discover => {
                leaflet.set_visible_child_name("content");
                view_stack.set_visible_child_name("discover");

                if phone_mode {
                    header_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_stack.set_visible_child_name("discover");
                    bottom_switcher_revealer.set_reveal_child(true);
                } else {
                    header_switcher_stack.set_visible_child_name("discover");
                    bottom_switcher_revealer.set_reveal_child(false);
                }
            }
            View::Library => {
                leaflet.set_visible_child_name("content");
                view_stack.set_visible_child_name("library");

                if phone_mode {
                    header_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_revealer.set_reveal_child(true);
                } else {
                    header_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_revealer.set_reveal_child(false);
                }
            }
            View::Player => {
                leaflet.set_visible_child_name("player");

                if phone_mode {
                    header_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_revealer.set_reveal_child(false);
                } else {
                    header_switcher_stack.set_visible_child_name("main");
                    bottom_switcher_revealer.set_reveal_child(false);
                }
            }
        }
    }
}
