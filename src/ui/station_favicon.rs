use gtk::prelude::*;
use gdk_pixbuf::Pixbuf;
use gdk::ContextExt;
use cairo::Context;

use std::f64;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum FaviconSize{
    Mini = 46,
    Small = 62,
    Big = 192,
}

pub struct StationFavicon {
    pub widget: gtk::Box,
    image: gtk::DrawingArea,
    stack: gtk::Stack,
    pixbuf: Rc<RefCell<Option<Pixbuf>>>,
    size: FaviconSize,
}

impl StationFavicon {
    pub fn new(size: FaviconSize) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_favicon.ui");
        get_widget!(builder, gtk::Box, station_favicon);
        get_widget!(builder, gtk::DrawingArea, image);
        get_widget!(builder, gtk::Stack, stack);
        get_widget!(builder, gtk::Image, placeholder);

        let pixbuf = Rc::new(RefCell::new(None));
        let ctx = station_favicon.get_style_context();

        match size{
            FaviconSize::Mini => {
                ctx.add_class("favicon-mini");
            },
            FaviconSize::Small => {
                ctx.add_class("favicon-small");
            },
            FaviconSize::Big => {
                ctx.add_class("favicon-big");
            },
        };
        image.set_size_request(size as i32, size as i32);
        placeholder.set_pixel_size(((size as i32) as f64 * 0.5) as i32);

        let favicon = Self {
            widget: station_favicon,
            image,
            stack,
            pixbuf,
            size,
        };

        favicon.setup_signals();
        favicon
    }

    pub fn set_pixbuf(&self, pixbuf: Pixbuf){
        *self.pixbuf.borrow_mut() = Some(pixbuf);
        self.image.queue_draw();
        self.stack.set_visible_child_name("image");
    }

    pub fn reset(&self){
        self.stack.set_visible_child_name("placeholder");
    }

    // Based on the custom drawing by GNOME Games
    // https://gitlab.gnome.org/GNOME/gnome-games/blob/de7e39e6c75423fe7357cdba48c1c3d73a2eea03/src/ui/savestate-listbox-row.vala#L106
    pub fn draw_image(image: &gtk::DrawingArea, cr: &Context, pixbuf: Rc<RefCell<Option<Pixbuf>>>, size: FaviconSize) -> gtk::Inhibit {
        let scale_factor = image.get_scale_factor() as f64;

    	let width = image.get_allocated_width();
		let height = image.get_allocated_height();

		let style = image.get_style_context();
		gtk::render_background(&style, cr, 0.0, 0.0, width.into(), height.into());
		gtk::render_frame(&style, cr, 0.0, 0.0, width.into(), height.into());

        match &*pixbuf.borrow() {
            Some(pixbuf) => {
                cr.save ();
		        cr.scale (1.0 / scale_factor, 1.0 / scale_factor);

		        let mask = Self::get_mask(image.clone(), size.clone());
		        let x_offset = (width as f64 * scale_factor - pixbuf.get_width() as f64) / 2.0;
		        let y_offset = (height as f64 * scale_factor - pixbuf.get_height() as f64) / 2.0;

		        cr.set_source_pixbuf(&pixbuf, x_offset, y_offset);
		        cr.mask_surface(&mask, 0.0, 0.0);
		        cr.restore();
		        gtk::Inhibit(false)
            },
            None => return gtk::Inhibit(false),
        }
    }

    fn get_mask (image: gtk::DrawingArea, size: FaviconSize) -> cairo::ImageSurface {
		let width = image.get_allocated_width() as f64;
		let height = image.get_allocated_height() as f64;
        let scale_factor = image.get_scale_factor() as f64;

		let mask = cairo::ImageSurface::create(cairo::Format::A8, (width * scale_factor) as i32, (height * scale_factor) as i32).unwrap();

        let mut border_radius = 8.0;
		if size == FaviconSize::Mini{
		    border_radius = 0.0;
		}

		let cr = Context::new(&mask);
		cr.scale(scale_factor.into(), scale_factor.into());
		Self::rounded_rectangle (cr.clone(), 0.0, 0.0, width, height, border_radius, size.clone());
		cr.fill ();

		return mask;
	}

	fn rounded_rectangle(cr: Context, x: f64, y: f64, width: f64, height: f64, radius: f64, size: FaviconSize) {
		let arc0: f64 = 0.0;
		let arc1: f64 = f64::consts::PI * 0.5;
		let arc2: f64 = f64::consts::PI;
		let arc3: f64 = f64::consts::PI * 1.5;

		cr.new_sub_path();

		// Don't render border radius on the right side for small favicons (used for station rows)
		if size == FaviconSize::Small {
			cr.arc(x + width, y,	      0.0, arc3, arc0);
		    cr.arc(x + width, y + height, 0.0, arc0, arc1);
		}else{
		    cr.arc(x + width - radius, y + radius,	        radius, arc3, arc0);
		    cr.arc(x + width - radius, y + height - radius, radius, arc0, arc1);
		}

		cr.arc(x + radius, y + height - radius, radius, arc1, arc2);
		cr.arc(x + radius, y + radius,          radius, arc2, arc3);
		cr.close_path();
	}

    fn setup_signals(&self) {
        let pixbuf = self.pixbuf.clone();
        let size = self.size.clone();
        self.image.connect_draw(move |dr, ctx|{
            Self::draw_image(dr, ctx, pixbuf.clone(), size.clone())
        });
    }
}
