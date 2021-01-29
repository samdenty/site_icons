# site_icons

An efficient website icon scraper for rust

```rust
use site_icons::Icons;

let icons = Icons::new();
// scrape the icons from a url
icons.load_website("https://github.com").await?;

// fetch all icons, ensuring they exist & determining size
let entries = icons.entries().await;
for icon in entries {
  println("{:?}", icon)
}
```

## Features

- Validates that all URLs exist and are actually images
- Determines the size of the icon by partially fetching it
- Supports WASM (and cloudflare workers)

### Sources

- HTML favicon tag (or looking for default `/favicon.ico`)
- [Web app manifest](https://developer.mozilla.org/en-US/docs/Web/Manifest) [`icons`](https://developer.mozilla.org/en-US/docs/Web/Manifest/icons) field
- `<img>` tags on the page, directly inside the header OR with a `src|alt|class` containing the text "logo"

## Running locally

Install [cargo make](https://github.com/sagiegurari/cargo-make) and then:

```bash
cargo make run https://github.com
```
