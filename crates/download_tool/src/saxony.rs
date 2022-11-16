use crate::{save_albedo, save_height, ImageGray, Tile};
use anyhow::{anyhow, Result};
use image::{io::Reader, DynamicImage, ImageFormat, Rgb, RgbImage};
use itertools::iproduct;
use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read},
};
use zip::ZipArchive;

pub(crate) fn gather_tiles(path: &str) -> Result<Vec<Tile>> {
    let file = File::open(path)?;
    let tiles = BufReader::new(file).lines().map(|name| {
        let coords = parse_coordinates(&name.unwrap()).unwrap();
        let dtm_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/DK9AshAQX7G1bsp/download?path=%2F&files=dgm1_33{}_{}_2_sn_xyz.zip", coords.0, coords.1);
        let dop_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/PEH5Gd2r0fPYV4r/download?path=%2F&files=dop20rgb_33{}_{}_2_sn_tiff.zip", coords.0, coords.1);
        let dsm_url = format!("https://geocloud.landesvermessung.sachsen.de/index.php/s/w7LQy9F6Yo3IxPp/download?path=%2F&files=dom1_33{}_{}_2_sn_xyz.zip", coords.0, coords.1);

        Tile::Saxony{dtm_url, dop_url, dsm_url}
    }).collect::<Vec<_>>();

    Ok(tiles)
}

pub(crate) fn parse_coordinates(name: &str) -> Result<(u32, u32)> {
    let parts = name.split('_').collect::<Vec<_>>();

    Ok((parts[1].parse::<u32>()? % 1000, parts[2].parse::<u32>()?))
}

pub(crate) async fn process_dtm(
    url: String,
    path: String,
    origin: (u32, u32),
    height: f32,
) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = ((coords.0 - origin.0) / 2, (origin.1 - coords.1) / 2);

    let buffer = download_file(url, ".xyz").await?;
    let image = parse_height(buffer, coords, height)?;
    save_height(&image, &path, "dtm", position)?;

    Ok(())
}

pub(crate) async fn process_dop(url: String, path: String, origin: (u32, u32)) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = ((coords.0 - origin.0) / 2, (origin.1 - coords.1) / 2);

    let buffer = download_file(url, ".tif").await?;
    let image = parse_albedo(buffer)?;
    save_albedo(&image, &path, "dop", position)?;

    Ok(())
}

pub(crate) async fn process_dsm(
    url: String,
    path: String,
    origin: (u32, u32),
    height: f32,
) -> Result<()> {
    let coords = parse_coordinates(&url)?;
    let position = ((coords.0 - origin.0) / 2, (origin.1 - coords.1) / 2);

    let buffer = download_file(url, ".xyz").await?;
    let image = parse_height(buffer, coords, height)?;
    save_height(&image, &path, "dsm", position)?;

    Ok(())
}

async fn download_file(path: String, extension: &str) -> Result<Vec<u8>> {
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

fn parse_height(buffer: Vec<u8>, coords: (u32, u32), max_height: f32) -> Result<DynamicImage> {
    let dimension: usize = 2000;
    let origin = (1000 * coords.0 as usize, 1000 * coords.1 as usize);

    let mut data = vec![0; (dimension * dimension) as usize];

    let reader = BufReader::new(Cursor::new(buffer));

    for coordinates in reader.lines().map(|line| {
        line.unwrap()
            .split_whitespace()
            .map(|value| value.parse::<f32>().unwrap())
            .collect::<Vec<_>>()
    }) {
        let x = coordinates[0] as usize - origin.0;
        let y = dimension - 1 - (coordinates[1] as usize - origin.1);
        let height = (coordinates[2] / max_height * u16::MAX as f32) as u16;

        data[y * dimension + x] = height;
    }

    Ok(DynamicImage::from(
        ImageGray::from_vec(dimension as u32, dimension as u32, data)
            .ok_or(anyhow!("Could not create image from the parsed data."))?,
    ))
}

fn parse_albedo(buffer: Vec<u8>) -> Result<DynamicImage> {
    let reader = Cursor::new(buffer);

    let mut reader = Reader::new(reader);
    reader.set_format(ImageFormat::Tiff);
    reader.no_limits();
    let image = reader.decode()?;
    let image = image.as_rgb8().unwrap();

    let size = image.width() / 5;

    let mut downscaled_image = RgbImage::new(size, size);

    for (x, y) in iproduct!(0..size, 0..size) {
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;

        for (dx, dy) in iproduct!(0..5, 0..5) {
            let pixel = image.get_pixel(x * 5 + dx, y * 5 + dy).0;

            r += pixel[0] as f32;
            g += pixel[1] as f32;
            b += pixel[2] as f32;
        }

        let pixel = Rgb([(r / 25.0) as u8, (g / 25.0) as u8, (b / 25.0) as u8]);
        downscaled_image.put_pixel(x, y, pixel);
    }

    Ok(DynamicImage::from(downscaled_image))
}
