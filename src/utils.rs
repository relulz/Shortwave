use gio::{self, ActionMapExt};
use glib::{self, object::WeakRef, Variant};
use gtk::prelude::*;

use crate::api::Station;

macro_rules! get_widget {
    ($builder:expr, $wtype:ty, $name:ident) => {
        let $name: $wtype = $builder.get_object(stringify!($name)).expect(&format!("Could not find widget \"{}\"", stringify!($name)));
    };
}

#[derive(Display, Debug, Clone, EnumString, PartialEq)]
pub enum Sorting {
    Default,
    Name,
    Language,
    Country,
    State,
    Codec,
    Votes,
    Bitrate,
}

#[derive(Display, Debug, Clone, EnumString, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}

pub fn station_cmp(a: &Station, b: &Station, sorting: Sorting, order: Order) -> std::cmp::Ordering {
    let mut station_a = a.clone();
    let mut station_b = b.clone();

    if order == Order::Descending {
        let tmp = station_a;
        station_a = station_b;
        station_b = tmp;
    }

    match sorting {
        Sorting::Default => std::cmp::Ordering::Equal,
        Sorting::Name => station_a.name.cmp(&station_b.name),
        Sorting::Language => station_a.language.cmp(&station_b.language),
        Sorting::Country => station_a.country.cmp(&station_b.country),
        Sorting::State => station_a.state.cmp(&station_b.state),
        Sorting::Codec => station_a.codec.cmp(&station_b.codec),
        Sorting::Votes => station_a.votes.cmp(&station_b.votes),
        Sorting::Bitrate => station_a.bitrate.cmp(&station_b.bitrate),
    }
}

// If you want to know more about lazy loading, you should read these:
// - https://en.wikipedia.org/wiki/Lazy_loading
// - https://blogs.gnome.org/ebassi/documentation/lazy-loading/comment-page-1/
//
// Source: gnome-podcasts (GPLv3)
// https://gitlab.gnome.org/World/podcasts/blob/7856b6fd27cb071583b87f55f3e47d9d8af9acb6/podcasts-gtk/src/utils.rs
pub(crate) fn lazy_load<T, C, F, W>(data: T, container: WeakRef<C>, mut contructor: F)
where
    T: IntoIterator + 'static,
    T::Item: 'static,
    C: IsA<glib::Object> + ContainerExt + 'static,
    F: FnMut(T::Item) -> W + 'static,
    W: IsA<gtk::Widget> + WidgetExt,
{
    let func = move |x| {
        let container = match container.upgrade() {
            Some(c) => c,
            None => return,
        };

        let widget = contructor(x);
        container.add(&widget);
        widget.show();
    };
    lazy_load_full(data, func);
}

pub(crate) fn lazy_load_full<T, F>(data: T, mut func: F)
where
    T: IntoIterator + 'static,
    T::Item: 'static,
    F: FnMut(T::Item) + 'static,
{
    let mut data = data.into_iter();
    gtk::idle_add(move || data.next().map(|x| func(x)).map(|_| glib::Continue(true)).unwrap_or_else(|| glib::Continue(false)));
}

// Removes special chars from a string
// If anybody knows a better way to do this, feel free to open a MR on GitLab :)
pub fn simplify_string(s: String) -> String {
    let mut string = s.replace("/", "");
    string = string.replace("\\", "");
    string = string.replace(":", "");
    string = string.replace("<", "");
    string = string.replace(">", "");
    string = string.replace("\"", "");
    string = string.replace("|", "");
    string = string.replace("?", "");
    string = string.replace("*", "");
    string = string.replace(".", "");
    string
}

// Creates an action named `name` in the action map `T with the handler `F`
// Stolen from gnome-podcasts
// https://gitlab.gnome.org/World/podcasts/blob/master/podcasts-gtk/src/window.rs
pub fn action<T, F>(thing: &T, name: &str, action: F)
where
    T: ActionMapExt,
    for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&Variant>) + 'static,
{
    // Create a stateless, parameterless action
    let act = gio::SimpleAction::new(name, None);
    // Connect the handler
    act.connect_activate(action);
    // Add it to the map
    thing.add_action(&act);
}
