use gtk::prelude::*;
use libhandy::PreferencesWindow;

use crate::settings::{SettingsManager, Key};

pub struct SettingsWindow {
    pub widget: PreferencesWindow,

    builder: gtk::Builder,
}

impl SettingsWindow {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/settings_window.ui");
        get_widget!(builder, PreferencesWindow, settings_window);

        settings_window.set_transient_for(Some(window));

        let window = Self {
            widget: settings_window,
            builder,
        };

        window.setup_signals();
        window
    }

    pub fn show(&self) {
        self.widget.set_visible(true);
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Switch, dark_mode_button);
        SettingsManager::bind_property(Key::DarkMode, &dark_mode_button, "active");
    }
}
