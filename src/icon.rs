use super::IconInfo;
use serde::Serialize;
use std::{
  cmp::Ordering,
  fmt::{self, Display},
};
use url::Url;

#[derive(Debug, Serialize, Clone, PartialOrd, PartialEq, Ord, Eq)]
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

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Icon {
  pub url: Url,
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
