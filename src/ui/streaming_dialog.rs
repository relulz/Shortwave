use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;

pub struct StreamingDialog {
    pub widget: libhandy::Dialog,

    sender: Sender<Action>,
    builder: gtk::Builder,
}

impl StreamingDialog {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/streaming_dialog.ui");
        get_widget!(builder, libhandy::Dialog, streaming_dialog);

        let sd = Self {
            widget: streaming_dialog,
            sender,
            builder,
        };

        sd.connect_signals();
        sd
    }

    pub fn show(&self){
        let application = self.builder.get_application().unwrap();
        let window = &application.get_windows()[0];
        self.widget.set_transient_for(Some(window));

        self.widget.set_visible(true);
        self.widget.show();
    }

    fn connect_signals(&self){
        // cancel_button
        let widget = self.widget.clone();
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(move |_|{
            widget.set_visible(false);
            widget.hide();
        });

        // hide on delete
        self.widget.connect_delete_event(|widget,_|{
            debug!("delete");
            widget.hide_on_delete();
            widget.hide();
            glib::signal::Inhibit(true)
        });
    }
}
