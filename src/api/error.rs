#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Serde error: {}", _0)]
    SerdeError(#[cause] serde_json::error::Error),
    #[fail(display = "URL parser error: {}", _0)]
    UrlParseError(#[cause] url::ParseError),
    #[fail(display = "GLib Error: {}", _0)]
    GLibError(#[cause] glib::error::Error),
    #[fail(display = "Input/Output error.")]
    IOError(#[cause] std::io::Error),
    #[fail(display = "Network error: {}", _0)]
    NetworkError(#[cause] isahc::Error),
    #[fail(display = "API error")]
    ApiError,
    #[fail(display = "Cache error")]
    CacheError,
}

// Maps a type to a variant of the Error enum
// Source: https://gitlab.gnome.org/World/podcasts/blob/945b40249cdf41d9c9766938f455e204ff88906e/podcasts-data/src/errors.rs#L94
macro_rules! easy_from_impl {
    ($outer_type:ty, $from:ty => $to:expr) => (
        impl From<$from> for $outer_type {
            fn from(err: $from) -> Self {
                $to(err)
            }
        }
    );
    ($outer_type:ty, $from:ty => $to:expr, $($f:ty => $t:expr),+) => (
        easy_from_impl!($outer_type, $from => $to);
        easy_from_impl!($outer_type, $($f => $t),+);
    );
}

easy_from_impl!(
    Error,
    serde_json::error::Error => Error::SerdeError,
    glib::error::Error       => Error::GLibError,
    url::ParseError          => Error::UrlParseError,
    std::io::Error           => Error::IOError,
    isahc::Error             => Error::NetworkError
);
