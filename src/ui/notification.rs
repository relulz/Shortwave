// Shortwave - notification.rs
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

use gtk::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Notification {
    revealer: gtk::Revealer,
    text_label: gtk::Label,
    error_label: gtk::Label,
    close_button: gtk::Button,
    error_box: gtk::Box,
}

impl Default for Notification {
    fn default() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/notification.ui");
        get_widget!(builder, gtk::Revealer, revealer);
        get_widget!(builder, gtk::Label, text_label);
        get_widget!(builder, gtk::Label, error_label);
        get_widget!(builder, gtk::Button, close_button);
        get_widget!(builder, gtk::Box, error_box);

        // Hide notification when close button gets clicked
        close_button.connect_clicked(clone!(@weak revealer => move |_| {
            revealer.set_reveal_child(false);
            Self::destroy(revealer);
        }));

        Self {
            revealer,
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

    fn destroy(_r: gtk::Revealer) {
        glib::source::timeout_add_seconds(1, move || {
            //TODO: r.destroy();
            glib::Continue(false)
        });
    }
}
