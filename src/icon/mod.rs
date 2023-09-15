mod icon_info;
mod icon_size;

pub use icon_info::*;
pub use icon_size::*;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::TryInto,
    error::Error,
    fmt::{self, Display},
    hash::{Hash, Hasher},
    str::FromStr,
};
use url::Url;

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq, SerializeDisplay, DeserializeFromStr)]
pub enum IconKind {
    AppIcon,
    SiteFavicon,
    SiteLogo,
}

impl Display for IconKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(match self {
            IconKind::SiteLogo => "site_logo",
            IconKind::AppIcon => "app_icon",
            IconKind::SiteFavicon => "site_favicon",
        })
    }
}

impl FromStr for IconKind {
    type Err = String;

    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        match kind {
            "site_logo" => Ok(IconKind::SiteLogo),
            "app_icon" => Ok(IconKind::AppIcon),
            "site_favicon" => Ok(IconKind::SiteFavicon),
            _ => Err("unknown icon kind!".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Icon {
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub kind: IconKind,
    #[serde(flatten)]
    pub info: IconInfo,
}

impl Hash for Icon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (
            &self.url,
            self.headers
                .iter()
                .sorted_by_key(|(key, _)| *key)
                .collect::<Vec<_>>(),
        )
            .hash(state);
    }
}

impl Icon {
    pub fn new(url: Url, kind: IconKind, info: IconInfo) -> Self {
        Icon::new_with_headers(url, HashMap::new(), kind, info)
    }

    pub fn new_with_headers(
        url: Url,
        headers: HashMap<String, String>,
        kind: IconKind,
        info: IconInfo,
    ) -> Self {
        Self {
            url,
            headers,
            kind,
            info,
        }
    }

    pub async fn load(
        url: Url,
        kind: IconKind,
        sizes: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        Icon::load_with_headers(url, HashMap::new(), kind, sizes).await
    }

    pub async fn load_with_headers(
        url: Url,
        headers: HashMap<String, String>,
        kind: IconKind,
        sizes: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        let info = IconInfo::load(url.clone(), (&headers).try_into().unwrap(), sizes).await?;

        Ok(Icon::new_with_headers(url, headers, kind, info))
    }
}

impl Ord for Icon {
    fn cmp(&self, other: &Self) -> Ordering {
        self.info.cmp(&other.info)
    }
}

impl PartialOrd for Icon {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
