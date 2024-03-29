#![allow(unused_imports)]
//! # site_icons
//! An efficient website icon scraper.
//!
//! ## Usage
//! ```rust
//! use site_icons::SiteIcons;
//!
//! async fn run() {
//! let mut icons = SiteIcons::new();
//! // scrape the icons from a url
//! let entries = icons.load_website("https://github.com", false).await.unwrap();
//!
//! // entries are sorted from highest to lowest resolution
//! for icon in entries {
//!   println!("{:?}", icon)
//! }
//! }
//! ```

#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate log;

#[macro_use]
mod utils;
mod html_parser;
mod icon;
mod icons;
mod manifest;

pub use icon::*;
pub use icons::*;

use once_cell::sync::Lazy;
use reqwest::{
  header::{HeaderMap, HeaderValue, USER_AGENT},
  Client,
};

static CLIENT: Lazy<Client> = Lazy::new(|| {
  let mut headers = HeaderMap::new();
  headers.insert(USER_AGENT, HeaderValue::from_str("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.104 Safari/537.36").unwrap());
  Client::builder().default_headers(headers).build().unwrap()
});
