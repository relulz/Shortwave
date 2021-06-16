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

use adw::subclass::prelude::*;
use adw::Carousel;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/de/haeckerfelix/Shortwave/gtk/featured_carousel.ui")]
    pub struct SwFeaturedCarousel {
        #[template_child]
        pub carousel: TemplateChild<Carousel>,
        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,
        pub pages: Rc<RefCell<Vec<gtk::Box>>>,
        pub visible_page: Rc<RefCell<usize>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SwFeaturedCarousel {
        const NAME: &'static str = "SwFeaturedCarousel";
        type ParentType = adw::Bin;
        type Type = super::SwFeaturedCarousel;

        fn new() -> Self {
            let pages = Rc::new(RefCell::new(Vec::new()));
            let visible_page = Rc::new(RefCell::new(0));

            Self {
                carousel: TemplateChild::default(),
                previous_button: TemplateChild::default(),
                next_button: TemplateChild::default(),
                pages,
                visible_page,
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SwFeaturedCarousel {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.init();
        }
    }

    impl WidgetImpl for SwFeaturedCarousel {}

    impl BinImpl for SwFeaturedCarousel {}
}

glib::wrapper! {
    pub struct SwFeaturedCarousel(ObjectSubclass<imp::SwFeaturedCarousel>)
        @extends gtk::Widget, adw::Bin;
}

impl SwFeaturedCarousel {
    pub fn init(&self) {
        self.setup_signals();
    }

    pub fn add_page(&self, title: &str, rgb: &str, action: Option<Action>) {
        let imp = imp::SwFeaturedCarousel::from_instance(self);

        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Shortwave/gtk/featured_carousel_page.ui");
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

        page_box.add_css_class("banner");
        let style_ctx = page_box.style_context();
        style_ctx.add_provider(&css_provider, 600);

        imp.carousel.insert(&page_box, imp.carousel.n_pages().try_into().unwrap());
        imp.pages.borrow_mut().append(&mut vec![page_box]);
    }

    fn setup_signals(&self) {
        let imp = imp::SwFeaturedCarousel::from_instance(self);

        imp.previous_button.connect_clicked(clone!(@weak self as this => move |_|{
            let imp = imp::SwFeaturedCarousel::from_instance(&this);

            if *imp.visible_page.borrow() != 0 {
                imp.carousel.scroll_to(&imp.pages.borrow()[*imp.visible_page.borrow() -1]);
                *imp.visible_page.borrow_mut() -= 1;
            }else{
                imp.carousel.scroll_to(&imp.pages.borrow()[(imp.pages.borrow().len()-1)]);
                *imp.visible_page.borrow_mut() = imp.pages.borrow().len()-1;
            }
        }));

        imp.next_button.connect_clicked(clone!(@weak self as this => move |_|{
            let imp = imp::SwFeaturedCarousel::from_instance(&this);

            if (*imp.visible_page.borrow()+1) != imp.pages.borrow().len() {
                imp.carousel.scroll_to(&imp.pages.borrow()[*imp.visible_page.borrow() +1]);
                *imp.visible_page.borrow_mut() += 1;
            }else{
                imp.carousel.scroll_to(&imp.pages.borrow()[0]);
                *imp.visible_page.borrow_mut() = 0;
            }
        }));

        imp.carousel.connect_page_changed(clone!(@weak self as this => move |_, a|{
            let imp = imp::SwFeaturedCarousel::from_instance(&this);

            *imp.visible_page.borrow_mut() = a.try_into().unwrap();
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
