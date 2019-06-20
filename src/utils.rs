use glib::{self, object::WeakRef};
use glib::prelude::*;
use gtk::prelude::*;
use gio::prelude::*;

// If you want to know more about lazy loading, you should read these:
// - https://en.wikipedia.org/wiki/Lazy_loading
// - https://blogs.gnome.org/ebassi/documentation/lazy-loading/comment-page-1/
//
// Source: gnome-podcasts (GPLv3)
// https://gitlab.gnome.org/World/podcasts/blob/7856b6fd27cb071583b87f55f3e47d9d8af9acb6/podcasts-gtk/src/utils.rs
pub fn lazy_load<T, C, F, W>(
    data: T,
    container: WeakRef<C>,
    mut contructor: F,
    cancellable: gio::Cancellable,
) where
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
    lazy_load_full(data, func, cancellable);
}

pub fn lazy_load_full<T, F>(data: T, mut func: F, cancellable: gio::Cancellable)
where
    T: IntoIterator + 'static,
    T::Item: 'static,
    F: FnMut(T::Item) + 'static,
{
    let mut data = data.into_iter();
    gtk::idle_add(move || {
        // If the Cancellable got activated, cancel the lazy loading
        // by stopping this idle_add "loop".
        if cancellable.is_cancelled(){
            return glib::Continue(false);
        }

        data.next()
            .map(|x| func(x))
            .map(|_|  glib::Continue(true) )
            .unwrap_or_else(|| glib::Continue(false))
    });
}
