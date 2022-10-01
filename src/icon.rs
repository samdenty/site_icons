use super::IconInfo;
use serde::Serialize;
use std::{
  cmp::Ordering,
  collections::HashMap,
  fmt::{self, Display},
  str::FromStr,
};
use url::Url;

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum IconKind {
  SiteLogo,
  SiteFavicon,
  AppIcon,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Icon {
  pub url: Url,
  pub headers: HashMap<String, String>,
  #[serde(with = "serde_with::rust::display_fromstr")]
  pub kind: IconKind,
  #[serde(flatten)]
  pub info: IconInfo,
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
