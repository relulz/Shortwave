use glib::Sender;
use gtk::prelude::*;
use libhandy::{LeafletExt, ViewSwitcherBarExt};

use crate::app::Action;
use crate::config;
use crate::widgets::Notification;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Discover,
    Library,
    Playback,
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    pub stack_player_box: gtk::Box,
    pub sidebar_player_box: gtk::Box,
    pub library_box: gtk::Box,
    pub discover_box: gtk::Box,

    builder: gtk::Builder,
    menu_builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/window.ui");
        let menu_builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/menu.ui");

        let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
        let view_headerbar: gtk::HeaderBar = builder.get_object("view_headerbar").unwrap();
        view_headerbar.set_title(Some(config::NAME));
        window.set_title(config::NAME);

        let stack_player_box: gtk::Box = builder.get_object("stack_player_box").unwrap();
        let sidebar_player_box: gtk::Box = builder.get_object("sidebar_player_box").unwrap();
        let library_box: gtk::Box = builder.get_object("library_box").unwrap();
        let discover_box: gtk::Box = builder.get_object("discover_box").unwrap();

        let window = Self {
            widget: window,
            stack_player_box,
            sidebar_player_box,
            library_box,
            discover_box,
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
        let content_leaflet: libhandy::Leaflet = self.builder.get_object("content_leaflet").unwrap();
        let builder = self.builder.clone();
        let menu_builder = self.menu_builder.clone();
        content_leaflet.connect_property_fold_notify(move |_| {
            Self::update_view(None, builder.clone(), menu_builder.clone());
        });
    }

    pub fn show_notification(&self, text: String) {
        let notification = Notification::new(text.as_str());

        let overlay: gtk::Overlay = self.builder.get_object("overlay").unwrap();
        notification.show(&overlay);
    }

    // view: None -> Just update the size layout (phone/desktop) without switching between views
    fn update_view(view: Option<View>, builder: gtk::Builder, menu_builder: gtk::Builder) {
        let view_switcherbar: libhandy::ViewSwitcherBar = builder.get_object("view_switcherbar").unwrap();
        let view_stack: gtk::Stack = builder.get_object("view_stack").unwrap();
        let content_leaflet: libhandy::Leaflet = builder.get_object("content_leaflet").unwrap();
        let header_leaflet: libhandy::Leaflet = builder.get_object("header_leaflet").unwrap();
        let sorting_mbutton: gtk::ModelButton = menu_builder.get_object("sorting_mbutton").unwrap();
        let library_mbutton: gtk::ModelButton = menu_builder.get_object("library_mbutton").unwrap();
        let add_button: gtk::Button = builder.get_object("add_button").unwrap();
        let back_button: gtk::Button = builder.get_object("back_button").unwrap();
        let stack_player_box: gtk::Box = builder.get_object("stack_player_box").unwrap();
        let sidebar_player_box: gtk::Box = builder.get_object("sidebar_player_box").unwrap();

        // Determine if window is currently in phone mode
        let phone_mode = content_leaflet.get_property_folded();

        // Add player widget to the correct container (depends on the view mode)
        let player_widget = Self::get_player_widget(builder.clone());
        if !phone_mode {
            sidebar_player_box.add(&player_widget);
        } else {
            stack_player_box.add(&player_widget);
        }

        view.map(|view| {
            // Determine if current visible page is library view
            let library_mode = view == View::Library;

            // show or hide buttons depending on the selected view
            add_button.set_visible(library_mode);
            back_button.set_visible(!library_mode);
            sorting_mbutton.set_sensitive(library_mode);
            library_mbutton.set_sensitive(library_mode);

            // Show requested view / page
            match view {
                View::Discover => {
                    content_leaflet.set_visible_child_name("content");
                    header_leaflet.set_visible_child_name("content");
                    view_stack.set_visible_child_name("discover");
                }
                View::Library => {
                    content_leaflet.set_visible_child_name("content");
                    header_leaflet.set_visible_child_name("content");
                    view_stack.set_visible_child_name("library");
                }
                View::Playback => {
                    if phone_mode {
                        view_stack.set_visible_child_name("playback");
                    } else {
                        content_leaflet.set_visible_child_name("playback");
                        header_leaflet.set_visible_child_name("playback");
                    }
                }
            }
        });

        // Show or hide bottom view switcher
        let show_view_switcher = phone_mode && view_stack.get_visible_child_name() != Some(glib::GString::from("discover"));
        view_switcherbar.set_reveal(show_view_switcher);
    }

    // Remove player widget from its parent and return it (to reparent it)
    fn get_player_widget(builder: gtk::Builder) -> gtk::Widget {
        let stack_player_box: gtk::Box = builder.get_object("stack_player_box").unwrap();
        let sidebar_player_box: gtk::Box = builder.get_object("sidebar_player_box").unwrap();

        let mut player_widget;
        let sidebar_player_widgets = sidebar_player_box.get_children();
        let stack_player_widgets = stack_player_box.get_children();
        if sidebar_player_widgets.is_empty() {
            stack_player_box.remove(&stack_player_widgets[0].clone());
            player_widget = stack_player_widgets[0].clone();
        } else {
            sidebar_player_box.remove(&sidebar_player_widgets[0].clone());
            player_widget = sidebar_player_widgets[0].clone();
        }

        player_widget
    }

    pub fn set_view(&self, view: View) {
        Self::update_view(Some(view), self.builder.clone(), self.menu_builder.clone());
    }
}
