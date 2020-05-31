// Shortwave - import_dialog.rs
// Copyright (C) 2020  Felix Häcker <haeckerfelix@gnome.org>
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

use gio::FileExt;
use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::i18n::*;

pub fn import_gradio_db(sender: Sender<Action>, _window: gtk::ApplicationWindow) {
    let import_dialog = gtk::FileChooserDialog::with_buttons::<gtk::Window>(
        Some(&i18n("Select database to import")),
        None,
        gtk::FileChooserAction::Open,
        &[(&i18n("Cancel"), gtk::ResponseType::Cancel), (&i18n("Import"), gtk::ResponseType::Accept)],
    );

    // For some reason we cannot access sqlite3 databases from the Flatpak sandbox,
    // so we're accessing the database directly by using "--filesystem=home"
    /*
    let import_dialog = gtk::FileChooserNative::new(
        Some(&i18n("Select database to import")),
        Some(&window),
        gtk::FileChooserAction::Open,
        Some(&i18n("Import")),
        Some(&i18n("Cancel")),
    );
    */

    // Set filechooser filters
    let filter = gtk::FileFilter::new();
    import_dialog.set_filter(&filter);
    filter.add_mime_type("application/x-sqlite3");
    filter.add_mime_type("application/vnd.sqlite3");

    if import_dialog.run() == gtk::ResponseType::Accept {
        let path = import_dialog.get_file().unwrap().get_path().unwrap();
        send!(sender, Action::LibraryImportGradioDatabase(path));
    }

    import_dialog.destroy();
}
