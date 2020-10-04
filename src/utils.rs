// Shortwave - utils.rs
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

use glib::{self, object::WeakRef};
use gtk::prelude::*;

use crate::api::Station;
use crate::i18n::*;

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
        std::mem::swap(&mut station_a, &mut station_b);
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

pub fn simplify_string(s: String) -> String {
    s.replace(&['/', '\0', '\\', ':', '<', '>', '\"', '|', '?', '*', '.'] as &[_], "")
}

pub fn station_subtitle(country: &str, state: &str, votes: i32) -> String {
    let mut string = if country != "" { country.to_string() } else { "".to_string() };

    if state != "" {
        string = format!("{} {}", string, state);
    }

    if string == "" {
        string = ni18n_f("{} Vote", "{} Votes", votes as u32, &[&votes.to_string()]);
    } else {
        string = ni18n_f("{} · {} Vote", "{} · {} Votes", votes as u32, &[&string, &votes.to_string()]);
    }

    string
}

// Removes all child items
pub fn remove_all_items<T>(container: &T)
where
    T: IsA<gtk::Container> + gtk::ContainerExt,
{
    let children = container.get_children();
    for widget in children {
        container.remove(&widget);
    }
}
