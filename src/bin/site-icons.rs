use clap::Clap;
use env_logger::Builder;
use log::LevelFilter;
use site_icons::Icons;
use std::error::Error;

#[derive(Clap)]
struct Opts {
  urls: Vec<String>,
  #[clap(long)]
  json: bool,
  #[clap(long)]
  /// Print out errors that occurred for skipped items
  debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let mut icons = Icons::new();
  let opts: Opts = Opts::parse();

  if opts.debug {
    let mut builder = Builder::new();
    builder.filter_module("info", LevelFilter::Info);
    builder.init();
  }

  for url in opts.urls {
    icons.load_website(&url).await?;
  }

  let entries = icons.entries().await;

  if opts.json {
    println!("{}", serde_json::to_string_pretty(&entries)?)
  } else {
    for icon in entries {
      println!("{} {} {}", icon.url, icon.kind, icon.info);
    }
  }

  Ok(())
}
