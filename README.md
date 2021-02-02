# site_icons

[![Crates.io](https://img.shields.io/crates/v/site_icons.svg)](https://crates.io/crates/site_icons)
[![Documentation](https://docs.rs/site_icons/badge.svg)](https://docs.rs/site_icons/)
![GitHub Sponsors](https://img.shields.io/github/sponsors/samdenty?style=social)

An efficient website icon scraper for rust or command line usage.

## Features

- Super fast!
- Partially downloads images to find the sizes
- Can extract a site logo `<img>` using a weighing system
- Works with inline-data URIs (and automatically converts `<svg>` to them)
- Supports WASM (and cloudflare workers)

### Command line usage

```bash
cargo install site_icons

site-icons https://github.com
# https://github.githubassets.com/favicons/favicon.svg site_favicon svg
# https://github.githubassets.com/app-icon-512.png app_icon png 512x512
# https://github.githubassets.com/app-icon-192.png app_icon png 192x192
# https://github.githubassets.com/apple-touch-icon-180x180.png app_icon png 180x180
```

### Rust usage

```rust
use site_icons::Icons;

let icons = Icons::new();
// scrape the icons from a url
icons.load_website("https://github.com").await?;

// fetch all icons, ensuring they exist & determining size
let entries = icons.entries().await;

// entries are sorted from highest to lowest resolution
for icon in entries {
  println("{:?}", icon)
}
```

### Sources

- HTML favicon tag (or looking for default `/favicon.ico`)
- [Web app manifest](https://developer.mozilla.org/en-US/docs/Web/Manifest) [`icons`](https://developer.mozilla.org/en-US/docs/Web/Manifest/icons) field
- `<img>` tags on the page, directly inside the header OR with a `src|alt|class` containing the text "logo"

## Running locally

Install [cargo make](https://github.com/sagiegurari/cargo-make) and then:

```bash
cargo make run https://github.com
```
