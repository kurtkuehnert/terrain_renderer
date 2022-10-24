mod parse;

use crate::parse::{parse_albedo, parse_height, save_albedo, save_height};
use anyhow::{anyhow, Result};
use futures::{future::join_all, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    io::{Cursor, Read},
};
use terrain_config::load_config;
use zip::ZipArchive;

fn parse_coordinates(name: &str) -> Result<(u32, u32)> {
    let parts = name.split('_').collect::<Vec<_>>();

    Ok((parts[1].parse::<u32>()? % 1000, parts[2].parse::<u32>()?))
}

async fn download_zip_file(path: String, extension: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(&path).await?;

    if response.status().is_client_error() {
        return Err(anyhow!("Does not exist."));
    }

    let bytes = response.bytes().await?;
    let reader = Cursor::new(bytes);

    let mut archive = ZipArchive::new(reader)?;

    let file_name = archive
        .file_names()
        .find(|name| name.ends_with(extension))
        .ok_or(anyhow!("File not found in archive."))?
        .to_string();

    let mut file = archive.by_name(&file_name)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

async fn process_dop(path: String, coords: (u32, u32), position: (u32, u32)) -> Result<()> {
    let dop_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/PEH5Gd2r0fPYV4r/download?path=%2F&files=dop20rgb_33{}_{}_2_sn_tiff.zip", coords.0, coords.1);
    let buffer = download_zip_file(dop_url, ".tif").await?;
    let image = parse_albedo(buffer)?;
    save_albedo(&image, &path, "dop", position)?;

    Ok(())
}

async fn process_dom(path: String, coords: (u32, u32), position: (u32, u32)) -> Result<()> {
    let dom_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/w7LQy9F6Yo3IxPp/download?path=%2F&files=dom1_33{}_{}_2_sn_xyz.zip", coords.0, coords.1);
    let buffer = download_zip_file(dom_url, ".xyz").await?;
    let image = parse_height(buffer, coords)?;
    save_height(&image, &path, "dom", position)?;

    Ok(())
}

async fn process_dgm(path: String, coords: (u32, u32), position: (u32, u32)) -> Result<()> {
    let dgm_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/DK9AshAQX7G1bsp/download?path=%2F&files=dgm1_33{}_{}_2_sn_xyz.zip", coords.0, coords.1);
    let buffer = download_zip_file(dgm_url, ".xyz").await?;
    let image = parse_height(buffer, coords)?;
    save_height(&image, &path, "dgm", position)?;

    Ok(())
}

async fn process_tile(
    url: String,
    path: String,
    origin: (u32, u32),
    bar: ProgressBar,
) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = ((coords.0 - origin.0) / 2, (origin.1 - coords.1) / 2);

    let task_dop = tokio::spawn(process_dop(path.clone(), coords, position));
    let task_dom = tokio::spawn(process_dom(path.clone(), coords, position));
    let task_dgm = tokio::spawn(process_dgm(path.clone(), coords, position));

    let tasks = join_all([task_dop, task_dom, task_dgm]).await;

    for task in tasks {
        task??;
    }

    bar.inc(1);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let path = config.terrain_path + "/source";
    let urls = config.urls;

    fs::create_dir_all(format!("{path}/dop"))?;
    fs::create_dir_all(format!("{path}/dom"))?;
    fs::create_dir_all(format!("{path}/dgm"))?;

    let mut origin = (u32::MAX, u32::MIN);

    for url in &urls {
        let coords = parse_coordinates(url)?;
        origin = (origin.0.min(coords.0), origin.1.max(coords.1));
    }

    println!("Started downloading and parsing {} tiles.", urls.len());
    let bar = ProgressBar::new(urls.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} tiles processed | Elapsed: {elapsed}, ETA: {eta}")?,
    );
    bar.tick();

    let results = futures::stream::iter(
        urls.iter()
            .map(|url| tokio::spawn(process_tile(url.clone(), path.clone(), origin, bar.clone()))),
    )
    .buffer_unordered(3)
    .collect::<Vec<_>>()
    .await;

    bar.finish();

    let mut failed_urls = Vec::new();
    let mut non_existing_urls = Vec::new();

    for (error, url) in results
        .into_iter()
        .zip(urls.iter())
        .map(|(result, url)| match result {
            Ok(result) => (result, url),
            Err(_) => (Err(anyhow!("Error while joining the task.")), url),
        })
        .filter_map(|(result, url)| match result {
            Ok(_) => None,
            Err(error) => Some((error, url)),
        })
    {
        if error.to_string() == "Does not exist." {
            non_existing_urls.push(url);
        } else {
            failed_urls.push(url);
            println!("{error}")
        }
    }

    println!(
        "\nFinished downloading and parsing {}/{} tiles successfully.",
        urls.len() - failed_urls.len() - non_existing_urls.len(),
        urls.len()
    );

    if !failed_urls.is_empty() {
        println!("\nThe following tiles failed to process.");

        for url in failed_urls {
            println!("{url}");
        }
    }

    if !non_existing_urls.is_empty() {
        println!("\nThe following tiles do not exist.");

        for url in non_existing_urls {
            println!("{url}");
        }
    }

    Ok(())
}
