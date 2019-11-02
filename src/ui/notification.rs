use gtk::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Notification {
    revealer: gtk::Revealer,
    spinner: gtk::Box,
    text_label: gtk::Label,
    error_label: gtk::Label,
    close_button: gtk::Button,
    error_box: gtk::Box,
}

impl Default for Notification {
    fn default() -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/notification.ui");
        get_widget!(builder, gtk::Revealer, revealer);
        get_widget!(builder, gtk::Box, spinner);
        get_widget!(builder, gtk::Label, text_label);
        get_widget!(builder, gtk::Label, error_label);
        get_widget!(builder, gtk::Button, close_button);
        get_widget!(builder, gtk::Box, error_box);

        // Hide notification when close button gets clicked
        let r = revealer.clone();
        close_button.connect_clicked(move |_| {
            r.set_reveal_child(false);
            Self::destroy(r.clone());
        });

        Self {
            revealer,
            spinner,
            text_label,
            error_label,
            close_button,
            error_box,
        }
    }
}

impl Notification {
    // Returns new information notification
    pub fn new_info(text: &str) -> Rc<Self> {
        let notification = Self::default();

        notification.text_label.set_text(text);
        notification.close_button.set_visible(true);

        Rc::new(notification)
    }

    // Returns new spinner notification
    pub fn new_spinner(text: &str) -> Rc<Self> {
        let notification = Self::default();

        notification.text_label.set_text(text);
        notification.spinner.set_visible(true);

        Rc::new(notification)
    }

    // Returns new error notification
    pub fn new_error(text: &str, error: &str) -> Rc<Self> {
        let notification = Self::default();

        notification.text_label.set_text(text);
        notification.error_label.set_text(error);
        notification.close_button.set_visible(true);
        notification.error_box.set_visible(true);

        Rc::new(notification)
    }

    pub fn show(&self, overlay: &gtk::Overlay) {
        overlay.add_overlay(&self.revealer);
        self.revealer.set_reveal_child(true);
    }

    pub fn hide(&self) {
        self.revealer.set_reveal_child(false);
        Self::destroy(self.revealer.clone());
    }

    fn destroy(r: gtk::Revealer) {
        gtk::timeout_add(1000, move || {
            r.destroy();
            glib::Continue(false)
        });
    }
}
