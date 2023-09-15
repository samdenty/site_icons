use crate::{html_parser, utils::push_url, Icon, IconKind, CLIENT};
use flo_stream::{MessagePublisher, Publisher, StreamPublisher};
use futures::future::{join_all, select_all};
use futures::prelude::*;
use futures::{join, StreamExt};
use itertools::Itertools;
use reqwest::{header::*, IntoUrl};
use std::convert::TryInto;
use std::error::Error;
use url::Url;
use vec1::Vec1;

pub struct SiteIcons {
    blacklist: Option<Box<dyn Fn(&Url) -> bool>>,
}

#[derive(Debug, Clone)]
enum LoadedKind {
    DefaultManifest(Option<Vec1<Icon>>),
    HeadTags(Option<Vec1<Icon>>),
    DefaultFavicon(Option<Icon>),
    SiteLogo(Option<Icon>),
}

impl SiteIcons {
    pub fn new() -> Self {
        SiteIcons { blacklist: None }
    }

    pub fn new_with_blacklist(blacklist: impl Fn(&Url) -> bool + 'static) -> Self {
        SiteIcons {
            blacklist: Some(Box::new(blacklist)),
        }
    }

    pub fn is_blacklisted(&self, url: &Url) -> bool {
        if let Some(is_blacklisted) = &self.blacklist {
            is_blacklisted(url)
        } else {
            false
        }
    }

    pub async fn load_website<U: IntoUrl>(
        &mut self,
        url: U,
        best_matches_only: bool,
    ) -> Result<Vec<Icon>, Box<dyn Error>> {
        let url = url.into_url()?;

        let manifest_urls = vec![
            push_url(&url, "manifest.json"),
            push_url(&url, "manifest.webmanifest"),
            url.join("/manifest.json")?,
            url.join("/manifest.webmanifest")?,
        ]
        .into_iter()
        .unique();

        let favicon_urls = vec![
            push_url(&url, "favicon.svg"),
            url.join("/favicon.svg")?,
            push_url(&url, "favicon.ico"),
            url.join("/favicon.ico")?,
        ]
        .into_iter()
        .unique();

        let html_response = async {
            let res = CLIENT
                .get(url.clone())
                .header(ACCEPT, "text/html")
                .send()
                .await
                .ok()?
                .error_for_status()
                .ok()?;

            let url = res.url().clone();

            if self.is_blacklisted(&url) {
                None
            } else {
                let body = res.bytes_stream().map(|res| {
                    res.map(|bytes| bytes.to_vec())
                        .map_err(|err| err.to_string())
                });

                let mut publisher = Publisher::new(128);
                let subscriber = publisher.subscribe();

                Some((
                    url,
                    async move { StreamPublisher::new(&mut publisher, body).await }.shared(),
                    subscriber,
                ))
            }
        }
        .shared();

        let mut futures = vec![
            async {
                let html_response = html_response.clone().await;

                LoadedKind::HeadTags(match html_response {
                    Some((url, _, body)) => html_parser::parse_head(&url, body)
                        .await
                        .ok()
                        .and_then(|icons| icons.try_into().ok()),
                    None => None,
                })
            }
            .boxed_local(),
            async {
                let html_response = html_response.clone().await;

                LoadedKind::SiteLogo(match html_response {
                    Some((url, complete, body)) => {
                        let (icons, _) = join!(
                            html_parser::parse_site_logo(&url, body, |url| self
                                .is_blacklisted(url)),
                            complete
                        );

                        icons.ok()
                    }
                    None => None,
                })
            }
            .boxed_local(),
            async {
                let manifests =
                    join_all(manifest_urls.map(|url| SiteIcons::load_manifest(url))).await;

                LoadedKind::DefaultManifest(
                    manifests
                        .into_iter()
                        .find_map(|manifest| manifest.ok().and_then(|icons| icons.try_into().ok())),
                )
            }
            .boxed_local(),
            async {
                let favicons = join_all(
                    favicon_urls.map(|url| Icon::load(url.clone(), IconKind::SiteFavicon, None)),
                )
                .await;

                LoadedKind::DefaultFavicon(favicons.into_iter().find_map(|favicon| favicon.ok()))
            }
            .boxed_local(),
        ];

        let mut icons: Vec<Icon> = Vec::new();
        let mut found_best_match = false;
        let mut previous_loads = Vec::new();

        while !futures.is_empty() {
            let (loaded, index, _) = select_all(&mut futures).await;
            futures.remove(index);

            match loaded.clone() {
                LoadedKind::DefaultManifest(manifest_icons) => {
                    if let Some(manifest_icons) = manifest_icons {
                        icons.extend(manifest_icons);
                        found_best_match = true;
                    }
                }
                LoadedKind::DefaultFavicon(favicon) => {
                    if let Some(favicon) = favicon {
                        icons.push(favicon);

                        if previous_loads
                            .iter()
                            .any(|kind| matches!(kind, LoadedKind::HeadTags(_)))
                        {
                            found_best_match = true;
                        }
                    }
                }
                LoadedKind::HeadTags(head_icons) => {
                    if let Some(head_icons) = head_icons {
                        icons.extend(head_icons);
                        found_best_match = true;
                    } else if previous_loads
                        .iter()
                        .any(|kind| matches!(kind, LoadedKind::DefaultFavicon(Some(_))))
                    {
                        found_best_match = true;
                    }
                }
                LoadedKind::SiteLogo(logo) => {
                    if let Some(logo) = logo {
                        icons.push(logo);
                    }
                }
            }

            previous_loads.push(loaded);

            icons.sort();
            icons = icons.into_iter().unique().collect();

            if best_matches_only && found_best_match {
                break;
            }
        }

        Ok(icons)
    }
}
