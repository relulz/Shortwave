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
use url::Url;

use crate::api::Client;
use crate::api::Error;
use crate::app::Action;
use crate::database::gradio_db;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;

pub async fn import_gradio_db(sender: Sender<Action>, window: gtk::ApplicationWindow) -> Result<(), Error> {
    let import_dialog = gtk::FileChooserNative::new(Some("Select database to import"), Some(&window), gtk::FileChooserAction::Open, Some("Import"), Some("Cancel"));

    // Set filechooser filters
    let filter = gtk::FileFilter::new();
    import_dialog.set_filter(&filter);
    filter.add_mime_type("application/x-sqlite3");
    filter.add_mime_type("application/vnd.sqlite3");

    if import_dialog.run() == gtk::ResponseType::Accept {
        let path = import_dialog.get_file().unwrap().get_path().unwrap();
        debug!("Import path: {:?}", path);

        // Get station identifiers
        let spinner_notification = Notification::new_spinner("Converting data...");
        send!(sender, Action::ViewShowNotification(spinner_notification.clone()));
        let ids = gradio_db::read_database(path).await?;
        spinner_notification.hide();

        // Get actual stations from identifiers
        let message = format!("Importing {} stations...", ids.len());
        let spinner_notification = Notification::new_spinner(&message);
        send!(sender, Action::ViewShowNotification(spinner_notification.clone()));

        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());
        let sender = sender.clone();
        let mut stations = Vec::new();
        for id in ids {
            let station = client.clone().get_station_by_identifier(id).await?;
            stations.insert(0, station);
        }

        spinner_notification.hide();
        send!(sender, Action::LibraryAddStations(stations.clone()));
        let message = format!("Imported {} stations!", stations.len());
        let notification = Notification::new_info(&message);
        send!(sender, Action::ViewShowNotification(notification));
    }

    import_dialog.destroy();
    Ok(())
}
