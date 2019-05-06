use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;

pub struct TagButton {
    pub widget: gtk::FlowBoxChild,
    name: String,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl TagButton {
    pub fn new(sender: Sender<Action>, title: &str, name: &str) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/tag_button.ui");
        let widget: gtk::FlowBoxChild = builder.get_object("tag_button").unwrap();

        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        title_label.set_text(title);

        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(
                format!(
                    ".tagbutton{{
                        background-color: grey;
                        background-image: url('resource://de/haeckerfelix/Shortwave/images/tags/{}.png');
                        background-size: cover;
                        color: white;
                    }}",
                    name
                )
                .as_bytes(),
            )
            .unwrap();

        let style_ctx = widget.get_style_context();
        style_ctx.add_class("tagbutton");
        style_ctx.add_provider(&css_provider, 600);

        let tb = Self {
            widget,
            name: name.to_string(),
            builder,
            sender,
        };

        tb.setup_signals();
        tb
    }

    fn setup_signals(&self) {
        let eventbox: gtk::EventBox = self.builder.get_object("eventbox").unwrap();
        let name = self.name.clone();
        eventbox.connect_button_press_event(move |_, _| {
            debug!("{} tag clicked", name);
            gtk::Inhibit(false)
        });
    }
}
