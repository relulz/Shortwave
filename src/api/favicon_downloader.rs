use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use gio::DataInputStream;
use url::Url;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

use crate::api::Error;
use crate::config;
use crate::path;

#[derive(Clone)]
pub struct FaviconDownloader {
    //session: Session,
}

impl FaviconDownloader {
    pub fn new() -> Self {
        let user_agent = format!("{}/{}", config::NAME, config::VERSION);

        //let session = soup::Session::new();
        //session.set_property_user_agent(Some(&user_agent));
        debug!("Initialized new soup session with user agent \"{}\"", user_agent);

        Self {}
    }

    pub async fn download(self, url: Url, size: i32) -> Result<Pixbuf, Error> {
        match self.get_cached_pixbuf(&url, &size).await {
            Ok(pixbuf) => return Ok(pixbuf),
            Err(_) => debug!("No cached favicon available for {:?}", url),
        }

        /*// Download pixbuf
        match soup::Message::new("GET", &url.to_string()) {
            Some(message) => {
                // Send created message
                let input_stream = self.session.send_async_future(&message).await?;

                // Create DataInputStream and read read received data
                let data_input_stream: DataInputStream = gio::DataInputStream::new(&input_stream);

                // Create pixbuf
                // We use 192px here, since that's the max size we're going to use
                let pixbuf = Pixbuf::new_from_stream_at_scale_async_future(&data_input_stream, 192, 192, true).await?;

                // Save pixbuf for caching
                let file = Self::get_file(&url)?;
                if !Self::exists(&file) {
                    let ios = file.create_readwrite_async_future(gio::FileCreateFlags::REPLACE_DESTINATION, glib::PRIORITY_DEFAULT).await?;
                    let data_output_stream = gio::DataOutputStream::new(&ios.get_output_stream().unwrap());
                    pixbuf.save_to_streamv_async_future(&data_output_stream, "png", &[]).await?;
                }
            }
            // Return error when message cannot be created
            None => return Err(Error::SoupMessageError),
        }

        Ok(self.get_cached_pixbuf(&url, &size).await?)*/
        Err(Error::CacheError)
    }

    async fn get_cached_pixbuf(&self, url: &Url, size: &i32) -> Result<Pixbuf, Error> {
        let file = Self::get_file(&url)?;
        if Self::exists(&file) {
            let ios = file.open_readwrite_async_future(glib::PRIORITY_DEFAULT).await.expect("Could not open file");
            let data_input_stream = DataInputStream::new(&ios.get_input_stream().unwrap());

            Ok(Pixbuf::new_from_stream_at_scale_async_future(&data_input_stream, *size, *size, true).await?)
        } else {
            Err(Error::CacheError)
        }
    }

    fn get_file(url: &Url) -> Result<gio::File, Error> {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        let mut path = path::CACHE.clone();
        path.push("favicons");
        std::fs::create_dir_all(path.as_path())?;

        path.push(hash.to_string());

        Ok(gio::File::new_for_path(&path))
    }

    fn exists(file: &gio::File) -> bool {
        let path = PathBuf::from(file.get_path().unwrap());
        path.exists()
    }
}
