use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use site_icons::SiteIcons;
use std::error::Error;

#[derive(Parser)]
struct Opts {
    url: String,

    #[clap(long)]
    fast: bool,
    #[clap(long)]
    json: bool,
    #[clap(long)]
    /// Print out errors that occurred for skipped items
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut icons = SiteIcons::new();
    let opts: Opts = Opts::parse();

    if opts.debug {
        let mut builder = Builder::new();
        builder.filter_level(LevelFilter::Info);
        builder.init();
    }

    let entries = icons.load_website(opts.url, opts.fast).await?;

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&entries)?)
    } else {
        for icon in entries {
            println!("{} {} {}", icon.url, icon.kind, icon.info);
        }
    }

    Ok(())
}
