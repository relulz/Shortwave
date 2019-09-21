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

        let window: gtk::ApplicationWindow = get_widget!(builder, "window");
        let app_label: gtk::Label = get_widget!(builder, "app_label");
        window.set_title(config::NAME);
        app_label.set_text(config::NAME);

        let player_box: gtk::Box = get_widget!(builder, "player_box");
        let mini_controller_box: gtk::Box = get_widget!(builder, "mini_controller_box");
        let library_box: gtk::Box = get_widget!(builder, "library_box");
        let discover_box: gtk::Box = get_widget!(builder, "discover_box");

        let discover_bottom_switcher: libhandy::ViewSwitcherBar = get_widget!(builder, "discover_bottom_switcher");
        let discover_header_switcher: libhandy::ViewSwitcher = get_widget!(builder, "discover_header_switcher");

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
        let popover_menu: gtk::PopoverMenu = get_widget!(window.menu_builder, "popover_menu");
        let appmenu_button: gtk::MenuButton = get_widget!(window.builder, "appmenu_button");
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
        let add_button: gtk::Button = get_widget!(self.builder, "add_button");
        let sender = self.sender.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowDiscover).unwrap();
        });

        // back_button
        let back_button: gtk::Button = get_widget!(self.builder, "back_button");
        let sender = self.sender.clone();
        back_button.connect_clicked(move |_| {
            sender.send(Action::ViewShowLibrary).unwrap();
        });

        // leaflet
        let view_stack: gtk::Stack = get_widget!(self.builder, "view_stack");
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

        let leaflet: libhandy::Leaflet = get_widget!(self.builder, "leaflet");
        leaflet.connect_property_visible_child_name_notify(leaflet_closure.clone());
        leaflet.connect_property_fold_notify(leaflet_closure.clone());
    }

    pub fn show_notification(&self, notification: Rc<Notification>) {
        let overlay: gtk::Overlay = get_widget!(self.builder, "overlay");
        notification.show(&overlay);
    }

    fn update_view(view: View, builder: gtk::Builder, menu_builder: gtk::Builder) {
        let bottom_switcher_revealer: gtk::Revealer = get_widget!(builder, "bottom_switcher_revealer");
        let bottom_switcher_stack: gtk::Stack = get_widget!(builder, "bottom_switcher_stack");
        let header_switcher_stack: gtk::Stack = get_widget!(builder, "header_switcher_stack");
        let view_stack: gtk::Stack = get_widget!(builder, "view_stack");
        let leaflet: libhandy::Leaflet = get_widget!(builder, "leaflet");
        let sorting_mbutton: gtk::ModelButton = get_widget!(menu_builder, "sorting_mbutton");
        let library_mbutton: gtk::ModelButton = get_widget!(menu_builder, "library_mbutton");
        let add_button: gtk::Button = get_widget!(builder, "add_button");
        let back_button: gtk::Button = get_widget!(builder, "back_button");

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
