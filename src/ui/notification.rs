use gtk::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Notification {
    revealer: gtk::Revealer,
    spinner: gtk::Spinner,
    text_label: gtk::Label,
    close_button: gtk::Button,
}

impl Default for Notification{
    fn default() -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/notification.ui");

        let revealer: gtk::Revealer = builder.get_object("revealer").unwrap();
        let spinner: gtk::Spinner = builder.get_object("spinner").unwrap();
        let text_label: gtk::Label = builder.get_object("text_label").unwrap();
        let close_button: gtk::Button = builder.get_object("close_button").unwrap();

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
            close_button,
        }
    }
}

impl Notification {
    // Returns new information notification
    pub fn new_info (text: &str) -> Rc<Self> {
        let notification = Self::default();

        notification.text_label.set_text(text);
        notification.close_button.set_visible(true);

        Rc::new(notification)
    }

    // Returns new spinner notification
    pub fn new_spinner (text: &str) -> Rc<Self> {
        let notification = Self::default();

        notification.text_label.set_text(text);
        notification.spinner.set_visible(true);

        Rc::new(notification)
    }

    pub fn show(&self, overlay: &gtk::Overlay) {
        overlay.add_overlay(&self.revealer);
        self.revealer.set_reveal_child(true);
    }

    pub fn hide (&self){
        self.revealer.set_reveal_child(false);
        Self::destroy(self.revealer.clone());
    }

    fn destroy(r: gtk::Revealer){
        gtk::timeout_add(1000, move||{
            r.destroy();
            glib::Continue(false)
        });
    }
}
