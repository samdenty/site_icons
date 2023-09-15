use crate::{utils::encode_svg, Icon, IconKind};
use futures::{Stream, StreamExt};
use html5ever::{
    driver,
    tendril::{Tendril, TendrilSink},
};
use scraper::{ElementRef, Html};
use std::error::Error;
use std::iter;
use tldextract::TldOption;
use url::Url;

pub async fn parse_site_logo(
    url: &Url,
    mut body: impl Stream<Item = Result<Vec<u8>, String>> + Unpin,
    is_blacklisted: impl Fn(&Url) -> bool,
) -> Result<Icon, Box<dyn Error>> {
    let mut parser = driver::parse_document(Html::new_document(), Default::default());
    while let Some(data) = body.next().await {
        if let Ok(data) = Tendril::try_from_byte_slice(&data?) {
            parser.process(data)
        }
    }

    let document = parser.finish();

    let mut logos: Vec<_> =
        document
            .select(selector!(
                "a[href='/'] img, a[href='/'] svg",
                "header img, header svg",
                "img[src*=logo]",
                "img[alt*=logo], svg[alt*=logo]",
                "*[class*=logo] img, *[class*=logo] svg",
                "*[id*=logo] img, *[id*=logo] svg",
                "img[class*=logo], svg[class*=logo]",
                "img[id*=logo], svg[id*=logo]",
            ))
            .enumerate()
            .filter_map(|(i, elem_ref)| {
                let elem = elem_ref.value();
                let ancestors = elem_ref
                    .ancestors()
                    .filter_map(ElementRef::wrap)
                    .map(|elem_ref| elem_ref.value())
                    .collect::<Vec<_>>();

                let skip_classnames = regex!("menu|search");
                let should_skip = ancestors.iter().any(|ancestor| {
                    ancestor
                        .attr("class")
                        .map(|attr| skip_classnames.is_match(&attr.to_lowercase()))
                        .or_else(|| {
                            ancestor
                                .attr("id")
                                .map(|attr| skip_classnames.is_match(&attr.to_lowercase()))
                        })
                        .unwrap_or(false)
                });

                if should_skip {
                    return None;
                }

                let mut weight = 0;

                // if in the header
                if ancestors.iter().any(|element| element.name() == "header") {
                    weight += 2;
                }

                if i == 0 {
                    weight += 1;
                }

                let mentions = |attr_name, is_match: Box<dyn Fn(&str) -> bool>| {
                    ancestors.iter().chain(iter::once(&elem)).any(|ancestor| {
                        ancestor
                            .attr(attr_name)
                            .map(|attr| is_match(&attr.to_lowercase()))
                            .unwrap_or(false)
                    })
                };

                if mentions("href", Box::new(|attr| attr == "/")) {
                    weight += 5;
                };

                let mentions_logo = |attr_name| {
                    mentions(
                        attr_name,
                        Box::new(|attr| regex!("logo([^s]|$)").is_match(attr)),
                    )
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

                if let Some(site_name) = url
                    .domain()
                    .and_then(|domain| TldOption::default().build().extract(domain).unwrap().domain)
                {
                    // if the alt contains the site_name then highest priority
                    if site_name.to_lowercase().split('-').any(|segment| {
                        mentions("alt", Box::new(move |attr| attr.contains(segment)))
                    }) {
                        weight += 10;
                    }
                }

                let href = if elem.name() == "svg" {
                    Some(Url::parse(&encode_svg(&elem_ref.html())).unwrap())
                } else {
                    elem.attr("src").and_then(|href| url.join(href).ok())
                };

                if let Some(href) = &href {
                    if is_blacklisted(href) {
                        return None;
                    }
                }

                href.map(|href| (href, elem_ref, weight))
            })
            .collect();

    logos.sort_by(|(_, _, a_weight), (_, _, b_weight)| b_weight.cmp(a_weight));

    // prefer <img> over svg
    let mut prev_weight = None;
    for (href, elem_ref, weight) in &logos {
        if let Some(prev_weight) = prev_weight {
            if weight != prev_weight {
                break;
            }
        }
        prev_weight = Some(weight);

        if elem_ref.value().name() == "img" {
            return Icon::load(href.clone(), IconKind::SiteLogo, None).await;
        }
    }

    match logos.into_iter().next() {
        Some((href, _, _)) => Icon::load(href.clone(), IconKind::SiteLogo, None).await,
        None => Err("No site logo found".into()),
    }
}
