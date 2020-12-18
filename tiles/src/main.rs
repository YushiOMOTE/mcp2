use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;
use tiled::{parse_with_path, LayerData};

#[derive(Serialize, Deserialize)]
struct Tile {
    map: Vec<Vec<u32>>,
}

#[derive(StructOpt)]
struct Opt {
    /// Tile map
    #[structopt(name = "tmx_file")]
    tmx: PathBuf,
    /// Tileset file directory
    #[structopt(name = "tsx_dir")]
    tsx: PathBuf,
    /// Json output file
    #[structopt(name = "output")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let file =
        File::open(&opt.tmx).with_context(|| format!("couldn't open: {}", opt.tmx.display()))?;

    let reader = BufReader::new(file);
    let map = parse_with_path(reader, &opt.tsx)
        .with_context(|| format!("couldn't parse tile map: {}", opt.tmx.display()))?;

    if let Some(map) = map.layers.first() {
        if let LayerData::Finite(map) = &map.tiles {
            let tile = Tile {
                map: map
                    .iter()
                    .map(|v| v.iter().map(|v| v.gid).collect())
                    .collect(),
            };
            serde_json::to_writer(
                File::create(&opt.output).with_context(|| {
                    format!(
                        "couldn't write to the output file: {}",
                        opt.output.display()
                    )
                })?,
                &tile,
            )
            .with_context(|| format!("couldn't pack to json: {}", opt.output.display()))?;
        }
    }

    Ok(())
}
