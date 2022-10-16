use crate::{utils::encode_svg, Icon, IconInfo, IconKind, CLIENT};
use future::join_all;
use futures::StreamExt;
use futures::{prelude::*, task::noop_waker};
use html5ever::{
  driver,
  tendril::{Tendril, TendrilSink},
};
use reqwest::{header::*, IntoUrl};
use scraper::{ElementRef, Html};
use serde::Deserialize;
use std::convert::TryInto;
use std::task::Poll;
use std::{collections::HashMap, error::Error, pin::Pin, task::Context};
use url::Url;

pub struct Icons {
  entries: Vec<Icon>,
  pending_entries: HashMap<
    Url,
    (
      IconKind,
      HashMap<String, String>,
      Pin<Box<dyn Future<Output = Result<IconInfo, Box<dyn Error>>>>>,
    ),
  >,
}

fn add_icon_entry(
  entries: &mut Vec<Icon>,
  url: Url,
  headers: HashMap<String, String>,
  kind: IconKind,
  info: Result<IconInfo, Box<dyn Error>>,
) {
  match info {
    Ok(info) => entries.push(Icon {
      url,
      headers,
      kind,
      info,
    }),
    Err(_) => warn_err!(info, "failed to parse icon"),
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
  pub fn add_icon(&mut self, url: Url, kind: IconKind, sizes: Option<String>) {
    self.add_icon_with_headers(url, HashMap::new(), kind, sizes)
  }

  /// Add an icon URL and start fetching it,
  /// along with the specified headers
  pub fn add_icon_with_headers(
    &mut self,
    url: Url,
    headers: HashMap<String, String>,
    kind: IconKind,
    sizes: Option<String>,
  ) {
    // check to see if it already exists
    let mut entries = self.entries.iter_mut();
    if let Some(existing_kind) = self
      .pending_entries
      .get_mut(&url)
      .map(|(kind, _, _)| kind)
      .or_else(|| {
        entries.find_map(|icon| {
          if icon.url.eq(&url) {
            Some(&mut icon.kind)
          } else {
            None
          }
        })
      })
    {
      // if the kind is more important, replace it
      if &kind > existing_kind {
        *existing_kind = kind;
      }
      return;
    }

    let mut info = Box::pin(IconInfo::load(
      url.clone(),
      (&headers).try_into().unwrap(),
      sizes,
    ));

    // Start fetching the icon
    let noop_waker = noop_waker();
    let cx = &mut Context::from_waker(&noop_waker);
    match info.poll_unpin(cx) {
      Poll::Ready(info) => add_icon_entry(&mut self.entries, url, headers, kind, info),
      Poll::Pending => {
        self.pending_entries.insert(url, (kind, headers, info));
      }
    };
  }

  pub async fn load_website<U: IntoUrl>(&mut self, url: U) -> Result<(), Box<dyn Error>> {
    let res = CLIENT.get(url).header(ACCEPT, "text/html").send().await?;
    let url = res.url().clone();
    let mut body = res.bytes_stream();

    let mut parser = driver::parse_document(Html::new_document(), Default::default());
    while let Some(data) = body.next().await {
      if let Ok(data) = Tendril::try_from_byte_slice(&data?) {
        parser.process(data)
      }
    }
    let document = parser.finish();

    {
      let mut found_favicon = false;

      for elem_ref in document.select(selector!(
        "link[rel='icon']",
        "link[rel='shortcut icon']",
        "link[rel='apple-touch-icon']",
        "link[rel='apple-touch-icon-precomposed']"
      )) {
        let elem = elem_ref.value();
        if let Some(href) = elem.attr("href").and_then(|href| url.join(&href).ok()) {
          let rel = elem.attr("rel").unwrap();
          self.add_icon(
            href,
            if rel.contains("apple-touch-icon") {
              IconKind::AppIcon
            } else {
              IconKind::SiteFavicon
            },
            elem.attr("sizes").map(|sizes| sizes.into()),
          );

          found_favicon = true;
        };
      }

      // Check for default favicon.ico
      if !found_favicon {
        self.add_icon(
          url.join("/favicon.ico").unwrap(),
          IconKind::SiteFavicon,
          None,
        );
      }
    }

    {
      let mut logos: Vec<_> = document
        .select(selector!(
          "a[href='/'] img, a[href='/'] svg",
          "header img, header svg",
          "img[src*=logo]",
          "img[alt*=logo], svg[alt*=logo]",
          "img[class*=logo], svg[class*=logo]",
        ))
        .map(|elem_ref| {
          let elem = elem_ref.value();
          let mut weight = 0;

          // if in the header
          if elem_ref
            .ancestors()
            .map(ElementRef::wrap)
            .flatten()
            .any(|element| element.value().name() == "header")
          {
            weight += 2;
          }

          let mentions_logo = |attr_name| {
            elem
              .attr(attr_name)
              .map(|attr| regex!("logo([^s]|$)").is_match(&attr.to_lowercase()))
              .unwrap_or(false)
          };
          if mentions_logo("class") || mentions_logo("id") {
            weight += 3;
          }
          if mentions_logo("alt") {
            weight += 2;
          }
          if mentions_logo("src") {
            weight += 1;
          }

          (elem_ref, weight)
        })
        .collect();

      logos.sort_by(|(_, a_weight), (_, b_weight)| b_weight.cmp(a_weight));

      // prefer <img> over svg
      let mut prev_weight = None;
      for (i, (logo, weight)) in logos.iter().enumerate() {
        if let Some(prev_weight) = prev_weight {
          if weight != prev_weight {
            break;
          }
        }
        prev_weight = Some(weight);

        if logo.value().name() == "img" {
          let (logo, weight) = logos.remove(i);
          logos.insert(0, (logo, weight + 1));
          break;
        }
      }

      for (elem_ref, _) in logos {
        let elem = elem_ref.value();

        if elem.name() == "svg" {
          let data_uri = Url::parse(&encode_svg(&elem_ref.html())).unwrap();
          self.add_icon(data_uri, IconKind::SiteLogo, None);
          break;
        }

        if let Some(href) = elem_ref
          .value()
          .attr("src")
          .and_then(|href| url.join(&href).ok())
        {
          self.add_icon(href, IconKind::SiteLogo, None);
          break;
        };
      }
    }

    for elem_ref in document.select(selector!("link[rel='manifest']")) {
      if let Some(href) = elem_ref
        .value()
        .attr("href")
        .and_then(|href| url.join(&href).ok())
      {
        warn_err!(self.load_manifest(href).await, "failed to fetch manifest");
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

  /// Fetch all the icons. Ordered from highest to lowest resolution
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
      .map(|(url, (kind, headers, info))| ((url, headers, kind), info))
      .unzip();

    let mut urls = urls.into_iter();

    for info in join_all(infos).await {
      let (url, headers, kind) = urls.next().unwrap();
      add_icon_entry(&mut self.entries, url, headers, kind, info);
    }

    self.entries.sort();

    self.entries
  }
}
