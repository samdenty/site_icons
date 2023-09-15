#[macro_use]
mod macros;
mod background_poll;
mod svg_encoder;

pub use background_poll::*;
pub use macros::*;
pub use svg_encoder::*;

use url::Url;

pub fn push_url(url: &Url, segment: &str) -> Url {
    let mut url = url.clone();
    url.path_segments_mut().unwrap().push(segment);
    url
}
