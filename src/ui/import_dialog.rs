use futures_util::future::FutureExt;
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

    if gtk::ResponseType::from(import_dialog.run()) == gtk::ResponseType::Accept {
        let path = import_dialog.get_file().unwrap().get_path().unwrap();
        debug!("Import path: {:?}", path);

        // Get station identifiers
        let message = format!("Converting data...");
        let spinner_notification = Notification::new_spinner(&message);
        sender.send(Action::ViewShowNotification(spinner_notification.clone())).unwrap();
        let ids = gradio_db::read_database(path).await?;
        spinner_notification.hide();

        // Get actual stations from identifiers
        let message = format!("Importing {} stations...", ids.len());
        let spinner_notification = Notification::new_spinner(&message);
        sender.send(Action::ViewShowNotification(spinner_notification.clone())).unwrap();

        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());
        let sender = sender.clone();
        let stations = client.get_stations_by_identifiers(ids).await?;

        spinner_notification.hide();
        sender.send(Action::LibraryAddStations(stations.clone())).unwrap();
        let message = format!("Imported {} stations!", stations.len());
        let notification = Notification::new_info(&message);
        sender.send(Action::ViewShowNotification(notification)).unwrap();
    }

    import_dialog.destroy();
    Ok(())
}
