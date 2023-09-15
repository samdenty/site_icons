use crate::{Icon, IconKind, SiteIcons, CLIENT};
use cached::proc_macro::cached;
use futures::future::join_all;
use reqwest::IntoUrl;
use serde::Deserialize;
use std::error::Error;
use url::Url;

#[derive(Debug, Deserialize)]
struct ManifestIcon {
    src: String,
    sizes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    icons: Vec<ManifestIcon>,
}

impl SiteIcons {
    pub async fn load_manifest<U: IntoUrl>(url: U) -> Result<Vec<Icon>, Box<dyn Error>> {
        let url = url.into_url()?;

        Ok(load_manifest_cached(url).await?)
    }
}

#[cached(sync_writes = true)]
async fn load_manifest_cached(url: Url) -> Result<Vec<Icon>, String> {
    let url = &url;

    let manifest: Manifest = CLIENT
        .get(url.clone())
        .send()
        .await
        .map_err(|e| format!("{}: {:?}", url, e))?
        .error_for_status()
        .map_err(|e| format!("{}: {:?}", url, e))?
        .json()
        .await
        .map_err(|e| format!("{}: {:?}", url, e))?;

    Ok(join_all(manifest.icons.into_iter().map(|icon| async move {
        if let Ok(src) = url.join(&icon.src) {
            Icon::load(src, IconKind::AppIcon, icon.sizes).await.ok()
        } else {
            None
        }
    }))
    .await
    .into_iter()
    .flatten()
    .collect())
}
