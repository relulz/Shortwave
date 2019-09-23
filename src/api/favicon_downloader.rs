use soup::prelude::*;
use soup::Session;
use gio::DataInputStream;
use gdk_pixbuf::Pixbuf;

use crate::api::Error;
use crate::config;

#[derive(Clone)]
pub struct FaviconDownloader{
    session: Session,
}

impl FaviconDownloader{
    pub fn new() -> Self{
        let user_agent = format!("{}/{}", config::NAME, config::VERSION);

        let session = soup::Session::new();
        session.set_property_user_agent(Some(&user_agent));
        debug!("Initialized new soup session with user agent \"{}\"", user_agent);

        Self { session }
    }

    // TODO: use Url here instead of String
    pub async fn download_favicon(self, url: String, size: i32) -> Result<Pixbuf, Error>{
        match soup::Message::new("GET", &url.to_string()){
            Some(message) => {
                // Send created message
                let input_stream = self.session.send_async_future(&message).await?;

                // Create DataInputStream and read read received data
                let data_input_stream: DataInputStream = gio::DataInputStream::new(&input_stream);

                // Create pixbuf
                let pixbuf = Pixbuf::new_from_stream_at_scale_async_future(&data_input_stream, size, size, true).await?;
                Ok(pixbuf)
            },
            // Return error when message cannot be created
            None => Err(Error::SoupMessageError),
        }
    }
}
