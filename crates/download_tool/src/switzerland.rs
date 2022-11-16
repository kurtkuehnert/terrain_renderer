use crate::{save_albedo, save_height, ImageGray, Tile};
use anyhow::{anyhow, Result};
use image::{DynamicImage, RgbImage};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Cursor},
};
use tiff::decoder::{Decoder, DecodingResult};

pub(crate) fn gather_tiles(path_dtm: &str, path_dop: &str) -> Result<Vec<Tile>> {
    let dtm_urls = File::open(path_dtm)?;
    let dtm_urls = BufReader::new(dtm_urls)
        .lines()
        .map(|url| {
            let url = url.unwrap();
            (parse_coordinates(&url).unwrap(), url)
        })
        .collect::<HashMap<_, _>>();

    let dop_urls = File::open(path_dop)?;
    let dop_urls = BufReader::new(dop_urls).lines();

    let mut tiles = Vec::new();

    for dop_url in dop_urls {
        let dop_url = dop_url.unwrap();
        let coords = parse_coordinates(&dop_url).unwrap();
        if let Some(dtm_url) = dtm_urls.get(&coords) {
            tiles.push(Tile::Switzerland {
                dtm_url: dtm_url.clone(),
                dop_url,
            });
        }
    }

    Ok(tiles)
}

pub(crate) fn parse_coordinates(name: &str) -> Result<(u32, u32)> {
    let parts = name.split('_').collect::<Vec<_>>();
    let parts = parts[2].split('/').collect::<Vec<_>>();
    let parts = parts[0].split('-').collect::<Vec<_>>();

    Ok((parts[0].parse::<u32>()?, parts[1].parse::<u32>()?))
}

pub(crate) async fn process_dtm(
    url: String,
    path: String,
    origin: (u32, u32),
    height: f32,
) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = (coords.0 - origin.0, origin.1 - coords.1);

    let buffer = download_file(url).await?;
    let image = parse_height(buffer, height)?;
    save_height(&image, &path, "dtm", position)?;

    Ok(())
}

pub(crate) async fn process_dop(url: String, path: String, origin: (u32, u32)) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = (coords.0 - origin.0, origin.1 - coords.1);

    let buffer = download_file(url).await?;
    let image = parse_albedo(buffer)?;
    save_albedo(&image, &path, "dop", position)?;

    Ok(())
}

async fn download_file(path: String) -> Result<Vec<u8>> {
    let response = reqwest::get(&path).await?;

    if response.status().is_client_error() {
        return Err(anyhow!("Does not exist."));
    }

    let bytes = response.bytes().await?;

    Ok(bytes.to_vec())
}

fn parse_albedo(buffer: Vec<u8>) -> Result<DynamicImage> {
    let reader = BufReader::new(Cursor::new(buffer));
    let mut decoder = Decoder::new(reader)?;
    let (width, height) = decoder.dimensions()?;

    if let DecodingResult::U8(data) = decoder.read_image()? {
        Ok(DynamicImage::from(
            RgbImage::from_raw(width, height, data)
                .ok_or(anyhow!("Could not create image from the parsed data."))?,
        ))
    } else {
        Err(anyhow!("Incompatible image format."))
    }
}

fn parse_height(buffer: Vec<u8>, max_height: f32) -> Result<DynamicImage> {
    let reader = BufReader::new(Cursor::new(buffer));
    let mut decoder = Decoder::new(reader)?;
    let (width, height) = decoder.dimensions()?;

    if let DecodingResult::F32(data) = decoder.read_image()? {
        let data = data
            .into_iter()
            .map(|value| (value / (2.0 * max_height) * u16::MAX as f32) as u16)
            .collect();
        Ok(DynamicImage::from(
            ImageGray::from_raw(width, height, data)
                .ok_or(anyhow!("Could not create image from the parsed data."))?,
        ))
    } else {
        Err(anyhow!("Incompatible image format."))
    }
}
