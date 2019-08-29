mod object_wrapper;
mod song_model;

pub use object_wrapper::ObjectWrapper;
pub use song_model::SongModel;

#[derive(Clone, Debug)]
pub enum Sorting {
    Name,
    Language,
    Country,
    State,
    Codec,
    Votes,
    Bitrate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}
