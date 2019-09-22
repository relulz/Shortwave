use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;

pub struct TileButton {
    pub widget: gtk::FlowBoxChild,
    image_name: String,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl TileButton {
    pub fn new(sender: Sender<Action>, title: &str, image_name: &str) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/tile_button.ui");
        get_widget!(builder, gtk::FlowBoxChild, tile_button);

        get_widget!(builder, gtk::Label, title_label);
        title_label.set_text(title);

        let css_provider = gtk::CssProvider::new();
        css_provider
            .load_from_data(
                format!(
                    ".tilebutton{{
                        background-color: grey;
                        background-image: url('resource://de/haeckerfelix/Shortwave/images/{}.png');
                        background-size: cover;
                        color: white;
                    }}",
                    image_name
                )
                .as_bytes(),
            )
            .unwrap();

        let style_ctx = tile_button.get_style_context();
        style_ctx.add_class("tilebutton");
        style_ctx.add_provider(&css_provider, 600);

        let tb = Self {
            widget: tile_button,
            image_name: image_name.to_string(),
            builder,
            sender,
        };

        tb.setup_signals();
        tb
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::EventBox, eventbox);
        let name = self.image_name.clone();
        eventbox.connect_button_press_event(move |_, _| {
            debug!("{} tag clicked", name);
            gtk::Inhibit(false)
        });
    }
}
