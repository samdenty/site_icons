use crate::utils::poll_in_background;
use crate::Icon;
use crate::IconKind;
use crate::SiteIcons;
use futures::future::join_all;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use lol_html::{element, errors::RewritingError, HtmlRewriter, Settings};
use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Display},
};
use url::Url;

#[derive(Debug)]
struct EndOfHead {}

impl Display for EndOfHead {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Error for EndOfHead {}

pub async fn parse_head(
    url: &Url,
    mut body: impl Stream<Item = Result<Vec<u8>, String>> + Unpin,
) -> Result<Vec<Icon>, Box<dyn Error>> {
    let mut icons = Vec::new();
    let new_icons = RefCell::new(Vec::new());

    {
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    element!("head", |head| {
                        head.on_end_tag(|_| Err(Box::new(EndOfHead {})))?;
                        Ok(())
                    }),
                    element!("link[rel~='manifest']", |manifest| {
                        if let Some(href) = manifest
                            .get_attribute("href")
                            .and_then(|href| url.join(&href).ok())
                        {
                            new_icons.borrow_mut().push(
                                async {
                                    SiteIcons::load_manifest(href).await.unwrap_or(Vec::new())
                                }
                                .boxed_local()
                                .shared(),
                            )
                        }

                        Ok(())
                    }),
                    element!(
                        join_with!(
                            ",",
                            "link[rel~='icon']",
                            "link[rel~='apple-touch-icon']",
                            "link[rel~='apple-touch-icon-precomposed']"
                        ),
                        |link| {
                            let rel = link.get_attribute("rel").unwrap();

                            if let Some(href) = link
                                .get_attribute("href")
                                .and_then(|href| url.join(&href).ok())
                            {
                                let kind = if rel.contains("apple-touch-icon") {
                                    IconKind::AppIcon
                                } else {
                                    IconKind::SiteFavicon
                                };

                                let sizes = link.get_attribute("sizes");

                                new_icons.borrow_mut().push(
                                    async {
                                        Icon::load(href, kind, sizes)
                                            .await
                                            .map(|icon| vec![icon])
                                            .unwrap_or(Vec::new())
                                    }
                                    .boxed_local()
                                    .shared(),
                                )
                            };

                            Ok(())
                        }
                    ),
                ],
                ..Settings::default()
            },
            |_: &[u8]| {},
        );

        while let Some(data) = poll_in_background(body.next(), join_all(icons.clone())).await {
            let result = rewriter.write(&data?);

            icons.extend(new_icons.borrow_mut().drain(..));

            match result {
                Err(RewritingError::ContentHandlerError(result)) => {
                    match result.downcast::<EndOfHead>() {
                        Ok(_) => break,
                        Err(err) => return Err(err),
                    };
                }

                result => result?,
            }
        }
    }

    let icons = join_all(icons).await.into_iter().flatten().collect();

    Ok(icons)
}
