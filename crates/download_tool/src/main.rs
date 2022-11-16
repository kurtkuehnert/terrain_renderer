mod saxony;
mod switzerland;

use anyhow::Result;
use bytemuck::cast_slice;
use dtm::DTM;
use futures::{future::join_all, StreamExt};
use image::{DynamicImage, ImageBuffer, Luma};
use indicatif::{ProgressBar, ProgressStyle};
use rapid_qoi::{Colors, Qoi};
use std::fs;
use terrain_settings::{load_settings, Dataset};

pub(crate) type ImageGray = ImageBuffer<Luma<u16>, Vec<u16>>;

#[derive(Clone, Debug)]
enum Tile {
    Switzerland {
        dtm_url: String,
        dop_url: String,
    },
    Saxony {
        dtm_url: String,
        dop_url: String,
        dsm_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    let path = settings.terrain_path.clone() + "/source";

    let tiles = match settings.dataset {
        Dataset::None => {
            panic!("No dataset selected.")
        }
        Dataset::Saxony { urls } => {
            fs::create_dir_all(format!("{path}/dtm"))?;
            fs::create_dir_all(format!("{path}/dop"))?;
            fs::create_dir_all(format!("{path}/dsm"))?;

            saxony::gather_tiles(&format!("{}/{}", settings.terrain_path, urls))
        }
        Dataset::Switzerland { urls_dtm, urls_dop } => {
            fs::create_dir_all(format!("{path}/dtm"))?;
            fs::create_dir_all(format!("{path}/dop"))?;

            switzerland::gather_tiles(
                &format!("{}/{}", settings.terrain_path, urls_dtm),
                &format!("{}/{}", settings.terrain_path, urls_dop),
            )
        }
    }?;

    let mut origin = (u32::MAX, u32::MIN);

    for tile in &tiles {
        let coords = match tile {
            Tile::Switzerland { dtm_url, .. } => switzerland::parse_coordinates(dtm_url)?,
            Tile::Saxony { dtm_url, .. } => saxony::parse_coordinates(dtm_url)?,
        };

        origin = (origin.0.min(coords.0), origin.1.max(coords.1));
    }

    println!("Started downloading and parsing {} tiles.", tiles.len());
    let bar = ProgressBar::new(tiles.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} tiles processed | Elapsed: {elapsed}, ETA: {eta}")?,
    );
    bar.tick();

    let results = futures::stream::iter(tiles.iter().map(|tile| {
        tokio::spawn(process_tile(
            tile.clone(),
            path.clone(),
            origin,
            settings.height,
            bar.clone(),
        ))
    }))
    .buffer_unordered(settings.parallel_downloads)
    .collect::<Vec<_>>()
    .await;

    bar.finish();

    let mut failed_tiles = Vec::new();

    for (result, url) in results.into_iter().zip(tiles.iter()) {
        match result {
            Ok(result) => {
                if let Err(error) = result {
                    failed_tiles.push(url);
                    println!("{error}");
                }
            }

            Err(error) => {
                failed_tiles.push(url);
                println!("{error}");
            }
        }
    }

    println!(
        "\nFinished downloading and parsing {}/{} tiles successfully.",
        tiles.len() - failed_tiles.len(),
        tiles.len()
    );

    if !failed_tiles.is_empty() {
        println!("\nThe following tiles failed to process.");

        for tile in failed_tiles {
            println!("{:?}", tile);
        }
    }

    Ok(())
}

async fn process_tile(
    tile: Tile,
    path: String,
    origin: (u32, u32),
    height: f32,
    bar: ProgressBar,
) -> Result<()> {
    let tasks = match tile {
        Tile::Switzerland { dtm_url, dop_url } => {
            let task_dtm = tokio::spawn(switzerland::process_dtm(
                dtm_url,
                path.clone(),
                origin,
                height,
            ));
            let task_dop = tokio::spawn(switzerland::process_dop(dop_url, path.clone(), origin));

            join_all([task_dop, task_dtm]).await
        }
        Tile::Saxony {
            dtm_url,
            dop_url,
            dsm_url,
        } => {
            let task_dtm = tokio::spawn(saxony::process_dtm(dtm_url, path.clone(), origin, height));
            let task_dop = tokio::spawn(saxony::process_dop(dop_url, path.clone(), origin));
            let task_dsm = tokio::spawn(saxony::process_dsm(dsm_url, path.clone(), origin, height));

            join_all([task_dtm, task_dop, task_dsm]).await
        }
    };

    for task in tasks {
        task??;
    }

    bar.inc(1);

    Ok(())
}

pub(crate) fn save_albedo(
    image: &DynamicImage,
    path: &str,
    name: &str,
    position: (u32, u32),
) -> Result<()> {
    let path = format!("{path}/{name}/{name}_{}_{}.qoi", position.0, position.1);

    let bytes = Qoi {
        width: image.width(),
        height: image.height(),
        colors: Colors::Rgb,
    }
    .encode_alloc(cast_slice(image.as_bytes()))?;

    fs::write(path, &bytes)?;

    Ok(())
}

pub(crate) fn save_height(
    image: &DynamicImage,
    path: &str,
    name: &str,
    position: (u32, u32),
) -> Result<()> {
    let path = format!("{path}/{name}/{name}_{}_{}.dtm", position.0, position.1);

    DTM {
        pixel_size: 2,
        channel_count: 1,
        width: image.width(),
        height: image.height(),
    }
    .encode_file(path, cast_slice(image.as_bytes()))?;

    Ok(())
}
