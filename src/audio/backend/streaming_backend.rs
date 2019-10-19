use glib::Sender;

use crate::app::Action;
use crate::ui::StreamingDialog;

pub struct StreamingBackend {
    pub streaming_dialog: StreamingDialog,

    sender: Sender<Action>,
}

impl StreamingBackend {
    pub fn new(sender: Sender<Action>) -> Self {
        let streaming_dialog = StreamingDialog::new(sender.clone());

        let streaming_backend = Self { streaming_dialog, sender };
        streaming_backend
    }

    pub fn open_dialog(&self){
        self.streaming_dialog.show();
    }
}
