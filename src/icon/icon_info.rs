use super::*;
use crate::CLIENT;
use data_url::DataUrl;
use futures::{io::Cursor, prelude::*, stream::TryStreamExt};
use mime::MediaType;
use reqwest::{header::*, Url};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    convert::TryFrom,
    error::Error,
    fmt::{self, Display},
    io,
};

enum IconKind {
    SVG,
    PNG,
    JPEG,
    ICO,
    GIF,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum IconInfo {
    PNG { size: IconSize },
    JPEG { size: IconSize },
    ICO { sizes: IconSizes },
    GIF { size: IconSize },
    SVG { size: Option<IconSize> },
}

impl IconInfo {
    async fn decode<R: AsyncRead + Unpin>(
        reader: &mut R,
        kind: Option<IconKind>,
    ) -> Result<IconInfo, Box<dyn Error>> {
        let mut header = [0; 2];
        reader.read_exact(&mut header).await?;

        match (kind, &header) {
            (Some(IconKind::SVG), bytes) => {
                let size = get_svg_size(bytes, reader).await?;
                Ok(IconInfo::SVG { size })
            }
            (_, &[0x60, byte_two]) => {
                let size = get_svg_size(&[0x60, byte_two], reader).await?;
                Ok(IconInfo::SVG { size })
            }
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
            (Some(IconKind::GIF), _) | (_, b"GI") => {
                let size = get_gif_size(reader).await?;
                Ok(IconInfo::GIF { size })
            }
            _ => Err(format!("unknown icon type ({:?})", header).into()),
        }
    }

    pub async fn load(
        url: Url,
        headers: HeaderMap,
        sizes: Option<String>,
    ) -> Result<IconInfo, Box<dyn Error>> {
        let sizes = sizes.as_ref().and_then(|s| IconSizes::try_from(s).ok());

        let (mime, mut body): (_, Box<dyn AsyncRead + Unpin>) = match url.scheme() {
            "data" => {
                let url = url.to_string();
                let url = DataUrl::process(&url).map_err(|_| "failed to parse data uri")?;

                let mime = url.mime_type().to_string().parse::<MediaType>()?;

                let body = Cursor::new(
                    url.decode_to_vec()
                        .map_err(|_| "failed to decode data uri body")?
                        .0,
                );

                (mime, Box::new(body))
            }

            _ => {
                let res = CLIENT
                    .get(url)
                    .headers(headers)
                    .send()
                    .await?
                    .error_for_status()?;

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
                        result.map_err(|error| {
                            io::Error::new(io::ErrorKind::Other, error.to_string())
                        })
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

            (mime::IMAGE, mime::GIF) => {
                if let Some(sizes) = sizes {
                    return Ok(IconInfo::GIF {
                        size: *sizes.largest(),
                    });
                }

                Some(IconKind::GIF)
            }

            (mime::IMAGE, mime::SVG) | (mime::TEXT, mime::PLAIN) => {
                if let Some(sizes) = sizes {
                    return Ok(IconInfo::SVG {
                        size: Some(*sizes.largest()),
                    });
                }

                Some(IconKind::SVG)
            }

            _ => None,
        };

        IconInfo::decode(&mut body, kind).await
    }

    pub fn size(&self) -> Option<&IconSize> {
        match self {
            IconInfo::ICO { sizes } => Some(sizes.largest()),
            IconInfo::PNG { size } | IconInfo::JPEG { size } | IconInfo::GIF { size } => Some(size),
            IconInfo::SVG { size } => size.as_ref(),
        }
    }

    pub fn sizes(&self) -> Option<IconSizes> {
        match self {
            IconInfo::ICO { sizes } => Some((*sizes).clone()),
            IconInfo::PNG { size } | IconInfo::JPEG { size } | IconInfo::GIF { size } => {
                Some((*size).into())
            }
            IconInfo::SVG { size } => size.map(|size| size.into()),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            IconInfo::PNG { .. } => "image/png",
            IconInfo::JPEG { .. } => "image/jpeg",
            IconInfo::ICO { .. } => "image/x-icon",
            IconInfo::GIF { .. } => "image/gif",
            IconInfo::SVG { .. } => "image/svg+xml",
        }
    }
}

impl Display for IconInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IconInfo::PNG { size } => write!(f, "png {}", size),
            IconInfo::JPEG { size } => write!(f, "jpeg {}", size),
            IconInfo::GIF { size } => write!(f, "gif {}", size),
            IconInfo::ICO { sizes } => write!(f, "ico {}", sizes),
            IconInfo::SVG { size } => {
                write!(
                    f,
                    "svg{}",
                    if let Some(size) = size {
                        format!(" {}", size)
                    } else {
                        "".to_string()
                    }
                )
            }
        }
    }
}

impl Ord for IconInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (IconInfo::SVG { size }, IconInfo::SVG { size: other_size }) => {
                match (size, other_size) {
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(size), Some(other_size)) => size.cmp(other_size),
                    (None, None) => Ordering::Equal,
                }
            }
            (IconInfo::SVG { .. }, _) => Ordering::Less,
            (_, IconInfo::SVG { .. }) => Ordering::Greater,

            _ => {
                let size = self.size().unwrap();
                let other_size = other.size().unwrap();

                size.cmp(other_size).then_with(|| match (self, other) {
                    (IconInfo::PNG { .. }, IconInfo::PNG { .. }) => Ordering::Equal,
                    (IconInfo::PNG { .. }, _) => Ordering::Less,
                    (_, IconInfo::PNG { .. }) => Ordering::Greater,

                    (IconInfo::GIF { .. }, IconInfo::GIF { .. }) => Ordering::Equal,
                    (IconInfo::GIF { .. }, _) => Ordering::Less,
                    (_, IconInfo::GIF { .. }) => Ordering::Greater,

                    (IconInfo::JPEG { .. }, IconInfo::JPEG { .. }) => Ordering::Equal,
                    (IconInfo::JPEG { .. }, _) => Ordering::Less,
                    (_, IconInfo::JPEG { .. }) => Ordering::Greater,

                    (IconInfo::ICO { .. }, IconInfo::ICO { .. }) => Ordering::Equal,
                    (IconInfo::ICO { .. }, _) => Ordering::Less,
                    (_, IconInfo::ICO { .. }) => Ordering::Greater,

                    _ => unreachable!(),
                })
            }
        }
    }
}

impl PartialOrd for IconInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
