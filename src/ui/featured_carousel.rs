// Shortwave - featured_carousel.rs
// Copyright (C) 2021  Felix Häcker <haeckerfelix@gnome.org>
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

use adw::Carousel;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

pub struct FeaturedCarousel {
    pub widget: gtk::Box,
    paginator: Carousel,

    pages: Rc<RefCell<Vec<gtk::Box>>>,
    visible_page: Rc<RefCell<usize>>,

    builder: gtk::Builder,
}

impl FeaturedCarousel {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/featured_carousel.ui");
        get_widget!(builder, gtk::Box, featured_carousel);
        get_widget!(builder, Carousel, paginator);

        let pages = Rc::new(RefCell::new(Vec::new()));
        let visible_page = Rc::new(RefCell::new(0));

        let carousel = Self {
            widget: featured_carousel,
            paginator,
            pages,
            visible_page,
            builder,
        };

        carousel.setup_signals();
        carousel
    }

    pub fn add_page(&self, title: &str, rgb: &str, action: Option<Action>) {
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/featured_carousel.ui");
        get_widget!(builder, gtk::Box, page_box);
        get_widget!(builder, gtk::Label, title_label);
        get_widget!(builder, gtk::Label, action_label);
        get_widget!(builder, gtk::Button, action_button);

        title_label.set_text(title);

        if let Some(a) = action {
            action_button.set_visible(true);
            action_button.set_action_name(Some(&a.name));
            action_label.set_text(&a.label);
        }

        // CSS styling
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(
            format!(
                ".banner{{
                        background-color: rgb({});
                    }}",
                rgb
            )
            .as_bytes(),
        );

        let style_ctx = page_box.get_style_context();
        style_ctx.add_class("banner");
        style_ctx.add_provider(&css_provider, 600);

        self.paginator.insert(&page_box, self.paginator.get_n_pages().try_into().unwrap());
        self.pages.borrow_mut().append(&mut vec![page_box]);
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Button, previous_button);
        previous_button.connect_clicked(
            clone!(@weak self.paginator as paginator, @weak self.pages as pages, @weak self.visible_page as visible_page => move |_|{
                if *visible_page.borrow() != 0 {
                    paginator.scroll_to(&pages.borrow()[*visible_page.borrow() -1]);
                    *visible_page.borrow_mut() -= 1;
                }else{
                    paginator.scroll_to(&pages.borrow()[(pages.borrow().len()-1)]);
                    *visible_page.borrow_mut() = pages.borrow().len()-1;
                }
            }),
        );

        get_widget!(self.builder, gtk::Button, next_button);
        next_button.connect_clicked(
            clone!(@weak self.paginator as paginator, @weak self.pages as pages, @weak self.visible_page as visible_page => move |_|{
                if (*visible_page.borrow()+1) != pages.borrow().len() {
                    paginator.scroll_to(&pages.borrow()[*visible_page.borrow() +1]);
                    *visible_page.borrow_mut() += 1;
                }else{
                    paginator.scroll_to(&pages.borrow()[0]);
                    *visible_page.borrow_mut() = 0;
                }
            }),
        );

        self.paginator.connect_page_changed(clone!(@weak self.visible_page as visible_page => move |_, a|{
            *visible_page.borrow_mut() = a.try_into().unwrap();
        }));
    }
}

pub struct Action {
    pub name: String,
    pub label: String,
}

impl Action {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            name: name.to_owned(),
            label: label.to_owned(),
        }
    }
}
