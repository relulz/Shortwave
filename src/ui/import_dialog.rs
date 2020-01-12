use futures_util::future::FutureExt;
use gio::FileExt;
use glib::Sender;
use gtk::prelude::*;
use url::Url;

use crate::api::Client;
use crate::app::Action;
use crate::database::gradio_db;
use crate::settings::{settings_manager, Key};
use crate::ui::Notification;

pub fn import_gradio_db(sender: Sender<Action>, window: gtk::ApplicationWindow) {
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
        let ids = gradio_db::read_database(path);
        let message = format!("Importing {} stations...", ids.len());
        let spinner_notification = Notification::new_spinner(&message);
        sender.send(Action::ViewShowNotification(spinner_notification.clone())).unwrap();

        // Get actual stations from identifiers
        let client = Client::new(Url::parse(&settings_manager::get_string(Key::ApiServer)).unwrap());
        let sender = sender.clone();
        let fut = client.get_stations_by_identifiers(ids).map(move |stations| {
            spinner_notification.hide();
            match stations {
                Ok(stations) => {
                    sender.send(Action::LibraryAddStations(stations.clone())).unwrap();

                    let message = format!("Imported {} stations!", stations.len());
                    let notification = Notification::new_info(&message);
                    sender.send(Action::ViewShowNotification(notification)).unwrap();
                }
                Err(err) => {
                    let notification = Notification::new_error("Could not receive station data.", &err.to_string());
                    sender.send(Action::ViewShowNotification(notification.clone())).unwrap();
                }
            }
        });

        let ctx = glib::MainContext::default();
        ctx.spawn_local(fut);
    }
    import_dialog.destroy();
}
