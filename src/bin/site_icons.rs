use clap::Clap;
use site_icons::Icons;
use std::error::Error;

#[derive(Clap)]
struct Opts {
  urls: Vec<String>,
  #[clap(long)]
  json: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let mut icons = Icons::new();
  let opts: Opts = Opts::parse();

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
