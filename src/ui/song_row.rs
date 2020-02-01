use chrono::NaiveTime;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;
use glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;
use open;

use crate::app::Action;
use crate::audio::Song;

pub struct SongRowPrivate {
    song: OnceCell<Song>,
    sender: OnceCell<Sender<Action>>,
    builder: gtk::Builder,
}

impl ObjectSubclass for SongRowPrivate {
    const NAME: &'static str = "SongRow";
    type ParentType = gtk::Box;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            song: OnceCell::new(),
            sender: OnceCell::new(),
            builder: gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_row.ui"),
        }
    }
}

// GLib.Object -> Gtk.Widget -> Gtk.Container -> Gtk.Box
impl ObjectImpl for SongRowPrivate {
    glib_object_impl!();
}
impl WidgetImpl for SongRowPrivate {}
impl ContainerImpl for SongRowPrivate {}
impl BoxImpl for SongRowPrivate {}

// Public part of the SongRow type. This behaves like a normal gtk-rs-style GObject binding
glib_wrapper! {
    pub struct SongRow(
        Object<subclass::simple::InstanceStruct<SongRowPrivate>,
        subclass::simple::ClassStruct<SongRowPrivate>,
        SongRowClass>)
        @extends gtk::Widget, gtk::Container, gtk::Box;

    match fn {
        get_type => || SongRowPrivate::get_type().to_glib(),
    }
}

impl SongRow {
    pub fn new(sender: Sender<Action>, song: Song) -> Self {
        let object = glib::Object::new(Self::static_type(), &[]).unwrap();
        let song_row = object.downcast::<SongRow>().expect("Wrong type");

        song_row.initialize(sender, song);
        song_row.setup_signals();
        song_row
    }

    fn initialize(&self, sender: Sender<Action>, song: Song) {
        let self_ = SongRowPrivate::from_instance(self);
        self_.song.set(song.clone()).unwrap();
        self_.sender.set(sender.clone()).unwrap();

        get_widget!(self_.builder, gtk::Box, song_row);
        get_widget!(self_.builder, gtk::Label, title_label);
        get_widget!(self_.builder, gtk::Label, duration_label);

        title_label.set_text(&song.title);
        title_label.set_tooltip_text(Some(song.title.as_str()));
        duration_label.set_text(&Self::format_duration(song.duration.as_secs()));
        duration_label.set_tooltip_text(Some(Self::format_duration(song.duration.as_secs()).as_str()));
        song_row.set_hexpand(false);

        self.add(&song_row);
        self.show_all();
    }

    fn setup_signals(&self) {
        let self_ = SongRowPrivate::from_instance(self);

        // Save button
        let sender = self_.sender.get().unwrap().clone();
        let song = self_.song.get().unwrap().clone();
        let widget = self.clone();
        get_widget!(self_.builder, gtk::Stack, button_stack);
        get_widget!(self_.builder, gtk::Button, save_button);
        save_button.connect_clicked(move |_| {
            sender.send(Action::PlaybackSaveSong(song.clone())).unwrap();

            // Show open button
            button_stack.set_visible_child_name("open");

            // Dim row
            let ctx = widget.get_style_context();
            ctx.add_class("dim-label");
        });

        // Open button
        let song = self_.song.get().unwrap().clone();
        get_widget!(self_.builder, gtk::Button, open_button);
        open_button.connect_clicked(move |_| {
            open::that(song.path.clone()).expect("Could not play song");
        });
    }

    // stolen from gnome-podcasts
    // https://gitlab.gnome.org/haecker-felix/podcasts/blob/2f8a6a91f87d7fa335a954bbaf2f70694f32f6dd/podcasts-gtk/src/widgets/player.rs#L168
    fn format_duration(seconds: u64) -> String {
        let time = NaiveTime::from_num_seconds_from_midnight(seconds as u32, 0);

        if seconds >= 3600 {
            time.format("%T").to_string()
        } else {
            time.format("%M∶%S").to_string()
        }
    }
}
