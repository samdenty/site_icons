use crate::{icon_size::*, CLIENT};
use data_url::DataUrl;
use futures::{io::Cursor, prelude::*, stream::TryStreamExt};
use mime::MediaType;
use reqwest::{header::*, Url};
use serde::{Deserialize, Serialize};
use std::{
  cmp::Ordering,
  error::Error,
  fmt::{self, Display},
  io,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum IconKind {
  PNG,
  JPEG,
  ICO,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum IconInfo {
  PNG { size: IconSize },
  JPEG { size: IconSize },
  ICO { sizes: IconSizes },
  SVG,
}

impl IconInfo {
  async fn decode<R: AsyncRead + Unpin>(
    reader: &mut R,
    kind: Option<IconKind>,
  ) -> Result<IconInfo, Box<dyn Error>> {
    let mut header = [0; 2];
    reader.read_exact(&mut header).await?;

    match (kind, &header) {
      (Some(IconKind::PNG), _) | (_, b"\x89P") => {
        let size = get_png_size(reader).await?;
        Ok(IconInfo::PNG { size })
      }
      (Some(IconKind::ICO), _) | (_, &[0x00, 0x00]) => {
        let sizes = get_ico_sizes(reader).await?;
        Ok(IconInfo::ICO { sizes })
      }
      (Some(IconKind::JPEG), _) | (_, &[0xFF, 0xD8]) => {
        let size = get_jpeg_size(reader).await?;
        Ok(IconInfo::JPEG { size })
      }
      _ => Err(format!("unknown icon type ({:?})", header).into()),
    }
  }

  pub async fn load(
    url: Url,
    headers: HeaderMap,
    sizes: Option<String>,
  ) -> Result<IconInfo, Box<dyn Error>> {
    let sizes = sizes.as_ref().and_then(|s| IconSizes::from_str(s).ok());

    let (mime, mut body): (_, Box<dyn AsyncRead + Unpin>) = match url.scheme() {
      "data" => {
        let url = url.to_string();
        let url = DataUrl::process(&url).map_err(|_| "failed to parse data uri")?;

        let mime = url.mime_type().to_string().parse::<MediaType>()?;

        let body = Cursor::new(
          url
            .decode_to_vec()
            .map_err(|_| "failed to decode data uri body")?
            .0,
        );

        (mime, Box::new(body))
      }

      _ => {
        let res = CLIENT.get(url).headers(headers).send().await?;
        if !res.status().is_success() {
          return Err("failed to fetch".into());
        };

        let mime = res
          .headers()
          .get(CONTENT_TYPE)
          .ok_or("no content type")?
          .to_str()?
          .parse::<MediaType>()?;

        let body = res
          .bytes_stream()
          .map(|result| {
            result.map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))
          })
          .into_async_read();

        (mime, Box::new(body))
      }
    };

    let kind = match (mime.type_(), mime.subtype()) {
      (mime::IMAGE, mime::PNG) => {
        if let Some(sizes) = sizes {
          return Ok(IconInfo::PNG {
            size: *sizes.largest(),
          });
        }
        Some(IconKind::PNG)
      }

      (mime::IMAGE, mime::JPEG) => {
        if let Some(sizes) = sizes {
          return Ok(IconInfo::JPEG {
            size: *sizes.largest(),
          });
        }
        Some(IconKind::JPEG)
      }

      (mime::IMAGE, "x-icon") | (mime::IMAGE, "vnd.microsoft.icon") => {
        if let Some(sizes) = sizes {
          return Ok(IconInfo::ICO { sizes });
        }

        Some(IconKind::ICO)
      }

      (mime::IMAGE, mime::SVG) | (mime::TEXT, mime::PLAIN) => return Ok(IconInfo::SVG),

      _ => None,
    };

    IconInfo::decode(&mut body, kind).await
  }

  pub fn size(&self) -> Option<&IconSize> {
    match self {
      IconInfo::ICO { sizes } => Some(sizes.largest()),
      IconInfo::PNG { size } | IconInfo::JPEG { size } => Some(size),
      IconInfo::SVG => None,
    }
  }

  pub fn sizes(&self) -> Option<IconSizes> {
    match self {
      IconInfo::ICO { sizes } => Some((*sizes).clone()),
      IconInfo::PNG { size } | IconInfo::JPEG { size } => Some((*size).into()),
      IconInfo::SVG => None,
    }
  }
}

impl Display for IconInfo {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      IconInfo::PNG { size } => write!(f, "png {}", size),
      IconInfo::JPEG { size } => write!(f, "jpeg {}", size),
      IconInfo::ICO { sizes } => write!(f, "ico {}", sizes),
      IconInfo::SVG => write!(f, "svg"),
    }
  }
}

impl Ord for IconInfo {
  fn cmp(&self, other: &Self) -> Ordering {
    let this_size = self.size();
    let other_size = other.size();

    if this_size.is_none() && other_size.is_none() {
      Ordering::Equal
    } else if let (Some(this_size), Some(other_size)) = (this_size, other_size) {
      this_size.cmp(other_size)
    } else if this_size.is_none() {
      Ordering::Less
    } else {
      Ordering::Greater
    }
  }
}

impl PartialOrd for IconInfo {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
