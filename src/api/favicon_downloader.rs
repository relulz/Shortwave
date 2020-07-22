// Shortwave - favicon_downloader.rs
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

use async_std::io::ReadExt;
use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use gio::DataInputStream;
use url::Url;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use crate::api::client::HTTP_CLIENT;
use crate::api::Error;
use crate::path;

pub struct FaviconDownloader {}

impl FaviconDownloader {
    pub async fn download(url: Url, size: i32) -> Result<Pixbuf, Error> {
        match Self::get_cached_pixbuf(&url, size).await {
            Ok(pixbuf) => return Ok(pixbuf),
            Err(_) => debug!("No cached favicon available for {:?}", url),
        }

        // We currently don't support "data:image/png" urls
        if url.scheme() == "data" {
            debug!("Unsupported favicon type for {:?}", url);
            return Err(Error::CacheError);
        }

        // Download favicon
        let mut bytes = vec![];
        HTTP_CLIENT.get_async(url.as_str()).await?.into_body().read_to_end(&mut bytes).await?;

        let input_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
        let pixbuf = Pixbuf::from_stream_at_scale_async_future(&input_stream, size, size, true).await?;

        // Write downloaded bytes into file
        let file = Self::get_file(&url)?;
        file.replace_contents_async_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .expect("Could not write favicon data");

        Ok(pixbuf)
    }

    async fn get_cached_pixbuf(url: &Url, size: i32) -> Result<Pixbuf, Error> {
        let file = Self::get_file(&url)?;
        if Self::exists(&file) {
            let ios = file.open_readwrite_async_future(glib::PRIORITY_DEFAULT).await.expect("Could not open file");
            let data_input_stream = DataInputStream::new(&ios.get_input_stream().unwrap());

            Ok(Pixbuf::from_stream_at_scale_async_future(&data_input_stream, size, size, true).await?)
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
        let path = file.get_path().unwrap();
        path.exists()
    }
}
