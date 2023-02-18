use std::path::Path;

use clap::*;
use eq_maps::{map_items::MapItems, map_draw::map_draw};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap()]
    out: String,
    
    #[clap()]
    files: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let paths = args.files.iter().map(|file| Path::new(file));
    let map_items = MapItems::load_from_files(paths).await?;

    map_draw(&map_items, Path::new(&args.out))?;

    Ok(())
}
