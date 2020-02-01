use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use gio::DataInputStream;
use isahc::prelude::*;
use url::Url;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

use crate::api::Error;
use crate::path;

pub struct FaviconDownloader {}

impl FaviconDownloader {
    pub async fn download(url: Url, size: i32) -> Result<Pixbuf, Error> {
        match Self::get_cached_pixbuf(&url, &size).await {
            Ok(pixbuf) => return Ok(pixbuf),
            Err(_) => debug!("No cached favicon available for {:?}", url),
        }

        // We currently don't support "data:image/png" urls
        if url.scheme() == "data" {
            return Err(Error::CacheError);
        }

        // Download favicon
        let response = isahc::get_async(url.to_string()).await?.text_async().await?;
        let bytes: Vec<u8> = response.into_bytes().into();

        // Write downloaded bytes into file
        let file = Self::get_file(&url)?;
        file.replace_contents_async_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .expect("Could not write favicon data");

        // Open downloaded favicon as pixbuf
        Ok(Self::get_cached_pixbuf(&url, &size).await?)
    }

    async fn get_cached_pixbuf(url: &Url, size: &i32) -> Result<Pixbuf, Error> {
        let file = Self::get_file(&url)?;
        if Self::exists(&file) {
            let ios = file.open_readwrite_async_future(glib::PRIORITY_DEFAULT).await.expect("Could not open file");
            let data_input_stream = DataInputStream::new(&ios.get_input_stream().unwrap());

            Ok(Pixbuf::new_from_stream_at_scale_async_future(&data_input_stream, *size, *size, true).await?)
        } else {
            Err(Error::CacheError)
        }
    }

    pub fn get_file(url: &Url) -> Result<gio::File, Error> {
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
