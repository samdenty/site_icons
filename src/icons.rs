use crate::{selector, Icon, IconInfo, IconKind, CLIENT};
use future::join_all;
use futures::StreamExt;
use futures::{prelude::*, task::noop_waker};
use html5ever::{
  driver,
  tendril::{Tendril, TendrilSink},
};
use reqwest::{header::*, IntoUrl};
use scraper::Html;
use serde::Deserialize;
use std::task::Poll;
use std::{collections::HashMap, error::Error, pin::Pin, task::Context};
use url::Url;

pub struct Icons {
  entries: Vec<Icon>,
  pending_entries: HashMap<
    Url,
    (
      IconKind,
      Pin<Box<dyn Future<Output = Result<IconInfo, Box<dyn Error>>>>>,
    ),
  >,
}

fn add_icon_entry(
  entries: &mut Vec<Icon>,
  url: Url,
  kind: IconKind,
  info: Result<IconInfo, Box<dyn Error>>,
) {
  match info {
    Ok(info) => entries.push(Icon { url, kind, info }),
    Err(e) => {
      warn!("failed to parse icon: {}", e);
    }
  }
}

impl Icons {
  pub fn new() -> Self {
    Icons {
      entries: Vec::new(),
      pending_entries: HashMap::new(),
    }
  }

  /// Add an icon URL and start fetching it
  pub fn add_icon(
    &mut self,
    url: Url,
    kind: IconKind,
    sizes: Option<String>,
  ) -> Result<(), Box<dyn Error>> {
    // check to see if it already exists
    let mut entries = self.entries.iter_mut();
    if let Some(existing_kind) = self
      .pending_entries
      .get_mut(&url)
      .map(|(kind, _)| kind)
      .or_else(|| entries.find_map(|icon| (icon.url == url).then_some(&mut icon.kind)))
    {
      // if the kind is more important, replace it
      if &kind > existing_kind {
        *existing_kind = kind;
      }
      return Ok(());
    }

    let mut info = Box::pin(IconInfo::get(url.clone(), sizes));

    // Start fetching the icon
    let noop_waker = noop_waker();
    let cx = &mut Context::from_waker(&noop_waker);
    match info.poll_unpin(cx) {
      Poll::Ready(info) => add_icon_entry(&mut self.entries, url, kind, info),
      Poll::Pending => {
        self.pending_entries.insert(url, (kind, info));
      }
    };

    Ok(())
  }

  pub async fn load_website<U: IntoUrl>(&mut self, url: U) -> Result<(), Box<dyn Error>> {
    let res = CLIENT.get(url).header(ACCEPT, "text/html").send().await?;
    let url = res.url().clone();
    let mut body = res.bytes_stream();

    let mut parser = driver::parse_document(Html::new_document(), Default::default());
    while let Some(data) = body.next().await {
      let tendril = Tendril::try_from_byte_slice(&data?).map_err(|_| "failed to parse html")?;
      parser.process(tendril);
    }
    let document = parser.finish();

    {
      let mut found_favicon = false;

      for element_ref in document.select(selector!(
        "link[rel='icon']",
        "link[rel='shortcut icon']",
        "link[rel='apple-touch-icon']",
        "link[rel='apple-touch-icon-precomposed']"
      )) {
        let elem = element_ref.value();
        if let Some(href) = elem.attr("href").and_then(|href| url.join(&href).ok()) {
          if self
            .add_icon(
              href,
              IconKind::SiteFavicon,
              elem.attr("sizes").map(|sizes| sizes.into()),
            )
            .is_ok()
          {
            found_favicon = true;
          };
        };
      }

      // Check for default favicon.ico
      if !found_favicon {
        self.add_icon(url.join("/favicon.ico")?, IconKind::SiteFavicon, None)?;
      }
    }

    for element_ref in document.select(selector!(
      "header img",
      "img[src*=logo]",
      "img[alt*=logo]",
      "img[class*=logo]"
    )) {
      if let Some(href) = element_ref
        .value()
        .attr("src")
        .and_then(|href| url.join(&href).ok())
      {
        if self.add_icon(href, IconKind::SiteLogo, None).is_ok() {
          break;
        };
      };
    }

    for element_ref in document.select(selector!("link[rel='manifest']")) {
      if let Some(href) = element_ref
        .value()
        .attr("href")
        .and_then(|href| url.join(&href).ok())
      {
        self.load_manifest(href).await?;
      }
    }

    Ok(())
  }

  pub async fn load_manifest(&mut self, manifest_url: Url) -> Result<(), Box<dyn Error>> {
    #[derive(Deserialize)]
    struct ManifestIcon {
      src: String,
      sizes: Option<String>,
    }

    #[derive(Deserialize)]
    struct Manifest {
      icons: Option<Vec<ManifestIcon>>,
    }

    let manifest: Manifest = CLIENT
      .get(manifest_url.as_str())
      .send()
      .await?
      .json()
      .await?;

    if let Some(icons) = manifest.icons {
      for icon in icons {
        if let Ok(src) = manifest_url.join(&icon.src) {
          let _ = self.add_icon(src, IconKind::AppIcon, icon.sizes);
        }
      }
    }

    Ok(())
  }

  /// Fetch all the icons and return a list of them.
  ///
  /// List is ordered from highest resolution to lowest resolution
  ///
  /// ```
  /// # async fn run() {
  /// let icons = Icons::new();
  /// icons.load_website("https://github.com").await?;
  ///
  /// let entries = icons.entries().await;
  /// for icon in entries {
  ///   println("{:?}", icon)
  /// }
  /// ```
  pub async fn entries(mut self) -> Vec<Icon> {
    let (urls, infos): (Vec<_>, Vec<_>) = self
      .pending_entries
      .into_iter()
      .map(|(url, (kind, info))| ((url, kind), info))
      .unzip();

    let mut urls = urls.into_iter();

    for info in join_all(infos).await {
      let (url, kind) = urls.next().unwrap();
      add_icon_entry(&mut self.entries, url, kind, info);
    }

    self.entries.sort();

    self.entries
  }
}
