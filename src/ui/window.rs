use gio::prelude::SettingsExt;
use glib::Sender;
use gtk::prelude::*;
use libhandy::LeafletExt;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::Action;
use crate::config;
use crate::settings::{Key, SettingsManager};
use crate::ui::Notification;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Discover,
    Library,
    Player,
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    pub leaflet: libhandy::Leaflet,
    pub mini_controller_box: gtk::Box,
    pub library_box: gtk::Box,
    pub discover_box: gtk::Box,

    pub discover_bottom_switcher: libhandy::ViewSwitcherBar,
    pub discover_header_switcher: libhandy::ViewSwitcher,

    current_view: Rc<RefCell<View>>,

    builder: gtk::Builder,
    menu_builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/window.ui");
        let menu_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/menu/app_menu.ui");

        get_widget!(builder, gtk::ApplicationWindow, window);
        get_widget!(builder, gtk::Label, app_label);
        window.set_title(config::NAME);
        app_label.set_text(config::NAME);

        get_widget!(builder, libhandy::Leaflet, leaflet);
        get_widget!(builder, gtk::Box, mini_controller_box);
        get_widget!(builder, gtk::Box, library_box);
        get_widget!(builder, gtk::Box, discover_box);

        get_widget!(builder, libhandy::ViewSwitcherBar, discover_bottom_switcher);
        get_widget!(builder, libhandy::ViewSwitcher, discover_header_switcher);

        let current_view = Rc::new(RefCell::new(View::Library));

        let window = Self {
            widget: window,
            leaflet,
            mini_controller_box,
            library_box,
            discover_box,
            discover_bottom_switcher,
            discover_header_switcher,
            current_view,
            builder,
            menu_builder,
            sender,
        };

        // Appmenu / hamburger button
        get_widget!(window.menu_builder, gtk::PopoverMenu, popover_menu);
        get_widget!(window.builder, gtk::MenuButton, appmenu_button);
        appmenu_button.set_popover(Some(&popover_menu));

        // Devel style class
        if config::APP_ID.ends_with("Devel") {
            let ctx = window.widget.get_style_context();
            ctx.add_class("devel");
        }

        // Restore window geometry
        let width = SettingsManager::get_integer(Key::WindowWidth);
        let height = SettingsManager::get_integer(Key::WindowHeight);
        window.widget.resize(width, height);

        window.setup_signals();
        window
    }

    fn setup_signals(&self) {
        // dark mode
        let s = SettingsManager::get_settings();
        let gtk_s = gtk::Settings::get_default().unwrap();
        s.bind("dark-mode", &gtk_s, "gtk-application-prefer-dark-theme", gio::SettingsBindFlags::GET);

        // add_button
        get_widget!(self.builder, gtk::Button, add_button);
        let sender = self.sender.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowDiscover).unwrap();
        });

        // back_button
        get_widget!(self.builder, gtk::Button, back_button);
        let sender = self.sender.clone();
        back_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowLibrary).unwrap();
        });

        // leaflet
        get_widget!(self.builder, gtk::Stack, view_stack);
        let current_view = self.current_view.clone();
        let builder = self.builder.clone();
        let menu_builder = self.menu_builder.clone();
        let leaflet_closure = move |leaflet: &libhandy::Leaflet| {
            if leaflet.get_property_folded() && leaflet.get_visible_child_name().unwrap() == "player" {
                *current_view.borrow_mut() = View::Player;
            } else {
                let view = match view_stack.get_visible_child_name().unwrap().as_str() {
                    "discover" => View::Discover,
                    _ => View::Library,
                };
                *current_view.borrow_mut() = view;
            }
            Self::update_view(current_view.borrow().clone(), builder.clone(), menu_builder.clone());
        };
        get_widget!(self.builder, libhandy::Leaflet, leaflet);
        leaflet.connect_property_fold_notify(leaflet_closure.clone());

        // window gets closed
        self.widget.connect_delete_event(move |window, _| {
            debug!("Saving window geometry.");
            let width = window.get_size().0;
            let height = window.get_size().1;

            SettingsManager::set_integer(Key::WindowWidth, width);
            SettingsManager::set_integer(Key::WindowHeight, height);
            Inhibit(false)
        });
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        get_widget!(self.builder, gtk::Overlay, overlay);
        notification.show(&overlay);
    }

    fn update_view(view: View, builder: gtk::Builder, menu_builder: gtk::Builder) {
        get_widget!(builder, gtk::Revealer, bottom_switcher_revealer);
        get_widget!(builder, gtk::Stack, bottom_switcher_stack);
        get_widget!(builder, gtk::Stack, header_switcher_stack);
        get_widget!(builder, gtk::Stack, view_stack);
        get_widget!(builder, libhandy::Leaflet, leaflet);
        get_widget!(menu_builder, gtk::ModelButton, sorting_mbutton);
        get_widget!(menu_builder, gtk::ModelButton, library_mbutton);
        get_widget!(builder, gtk::Button, add_button);
        get_widget!(builder, gtk::Button, back_button);

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

    pub fn set_view(&self, view: View) {
        *self.current_view.borrow_mut() = view;
        Self::update_view(self.current_view.borrow().clone(), self.builder.clone(), self.menu_builder.clone());
    }
}
