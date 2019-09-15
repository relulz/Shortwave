use glib::Sender;
use gtk::prelude::*;
use libhandy::LeafletExt;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::Action;
use crate::config;
use crate::ui::Notification;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Discover,
    Library,
    Player,
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    pub player_box: gtk::Box,
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
        let menu_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/menu.ui");

        let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
        let app_label: gtk::Label = builder.get_object("app_label").unwrap();
        window.set_title(config::NAME);
        app_label.set_text(config::NAME);

        let player_box: gtk::Box = builder.get_object("player_box").unwrap();
        let mini_controller_box: gtk::Box = builder.get_object("mini_controller_box").unwrap();
        let library_box: gtk::Box = builder.get_object("library_box").unwrap();
        let discover_box: gtk::Box = builder.get_object("discover_box").unwrap();

        let discover_bottom_switcher: libhandy::ViewSwitcherBar = builder.get_object("discover_bottom_switcher").unwrap();
        let discover_header_switcher: libhandy::ViewSwitcher = builder.get_object("discover_header_switcher").unwrap();

        let current_view = Rc::new(RefCell::new(View::Library));

        let window = Self {
            widget: window,
            player_box,
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
        let popover_menu: gtk::PopoverMenu = window.menu_builder.get_object("popover_menu").unwrap();
        let appmenu_button: gtk::MenuButton = window.builder.get_object("appmenu_button").unwrap();
        appmenu_button.set_popover(Some(&popover_menu));

        // Devel style class
        if config::APP_ID.ends_with("Devel") {
            let ctx = window.widget.get_style_context();
            ctx.add_class("devel");
        }

        window.setup_signals();
        window
    }

    fn setup_signals(&self) {
        // add_button
        let add_button: gtk::Button = self.builder.get_object("add_button").unwrap();
        let sender = self.sender.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowDiscover).unwrap();
        });

        // back_button
        let back_button: gtk::Button = self.builder.get_object("back_button").unwrap();
        let sender = self.sender.clone();
        back_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowLibrary).unwrap();
        });

        // leaflet
        let view_stack: gtk::Stack = self.builder.get_object("view_stack").unwrap();
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

        let leaflet: libhandy::Leaflet = self.builder.get_object("leaflet").unwrap();
        leaflet.connect_property_visible_child_name_notify(leaflet_closure.clone());
        leaflet.connect_property_fold_notify(leaflet_closure.clone());
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        let overlay: gtk::Overlay = self.builder.get_object("overlay").unwrap();
        notification.show(&overlay);
    }

    fn update_view(view: View, builder: gtk::Builder, menu_builder: gtk::Builder) {
        let bottom_switcher_revealer: gtk::Revealer = builder.get_object("bottom_switcher_revealer").unwrap();
        let bottom_switcher_stack: gtk::Stack = builder.get_object("bottom_switcher_stack").unwrap();
        let header_switcher_stack: gtk::Stack = builder.get_object("header_switcher_stack").unwrap();
        let view_stack: gtk::Stack = builder.get_object("view_stack").unwrap();
        let leaflet: libhandy::Leaflet = builder.get_object("leaflet").unwrap();
        let sorting_mbutton: gtk::ModelButton = menu_builder.get_object("sorting_mbutton").unwrap();
        let library_mbutton: gtk::ModelButton = menu_builder.get_object("library_mbutton").unwrap();
        let add_button: gtk::Button = builder.get_object("add_button").unwrap();
        let back_button: gtk::Button = builder.get_object("back_button").unwrap();

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
