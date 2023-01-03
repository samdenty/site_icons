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

First install the binary:

```
cargo install site_icons
```

then run either:

<details open>
<summary>Text output:</summary>

<blockquote>

```bash
# site-icons https://github.com

https://github.githubassets.com/favicons/favicon.svg site_favicon svg
https://github.githubassets.com/app-icon-512.png app_icon png 512x512
https://github.githubassets.com/apple-touch-icon-180x180.png app_icon png 180x180
```

</blockquote>

</details>

<details open>
<summary>JSON output:</summary>

<blockquote>

```jsonc
// site-icons https://reactjs.org --json

[
  {
    "url": "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9Ii0xMS41IC0xMC4yMzE3NCAyMyAyMC40NjM0OCI+CiAgPHRpdGxlPlJlYWN0IExvZ288L3RpdGxlPgogIDxjaXJjbGUgY3g9IjAiIGN5PSIwIiByPSIyLjA1IiBmaWxsPSIjNjFkYWZiIi8+CiAgPGcgc3Ryb2tlPSIjNjFkYWZiIiBzdHJva2Utd2lkdGg9IjEiIGZpbGw9Im5vbmUiPgogICAgPGVsbGlwc2Ugcng9IjExIiByeT0iNC4yIi8+CiAgICA8ZWxsaXBzZSByeD0iMTEiIHJ5PSI0LjIiIHRyYW5zZm9ybT0icm90YXRlKDYwKSIvPgogICAgPGVsbGlwc2Ugcng9IjExIiByeT0iNC4yIiB0cmFuc2Zvcm09InJvdGF0ZSgxMjApIi8+CiAgPC9nPgo8L3N2Zz4K",
    "headers": {},
    "kind": "site_logo",
    "type": "svg",
    "size": null
  },
  {
    "url": "https://reactjs.org/icons/icon-512x512.png?v=f4d46f030265b4c48a05c999b8d93791",
    "headers": {},
    "kind": "app_icon",
    "type": "png",
    "size": "512x512"
  },
  {
    "url": "https://reactjs.org/favicon.ico",
    "headers": {},
    "kind": "site_favicon",
    "type": "ico",
    "sizes": ["64x64", "32x32", "24x24", "16x16"]
  },
  {
    "url": "https://reactjs.org/favicon-32x32.png?v=f4d46f030265b4c48a05c999b8d93791",
    "headers": {},
    "kind": "site_favicon",
    "type": "png",
    "size": "32x32"
  }
]
```

</blockquote>

</details>

### Sources

- HTML favicon tag (or looking for default `/favicon.svg` / `/favicon.ico`)
- [Web app manifest](https://developer.mozilla.org/en-US/docs/Web/Manifest) [`icons`](https://developer.mozilla.org/en-US/docs/Web/Manifest/icons) field
- `<img>` tags on the page, directly inside the header OR with a `src|alt|class` containing the text "logo"

## Running locally

```bash
git clone https://github.com/samdenty/site_icons
cd site_icons
cargo run https://github.com
```
