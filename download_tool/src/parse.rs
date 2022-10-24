use anyhow::{anyhow, Result};
use bytemuck::cast_slice;
use dtm::DTM;
use image::{io::Reader, DynamicImage, ImageBuffer, ImageFormat, Luma, Rgb, RgbImage};
use itertools::iproduct;
use rapid_qoi::{Colors, Qoi};
use std::{
    fs,
    io::{BufRead, BufReader, Cursor},
};

pub(crate) fn parse_height(buffer: Vec<u8>, coords: (u32, u32)) -> Result<DynamicImage> {
    let dimension: usize = 2000;
    let max_height: f64 = 1250.0;
    let origin = (1000 * coords.0 as usize, 1000 * coords.1 as usize);

    let mut data = vec![0; (dimension * dimension) as usize];

    let reader = BufReader::new(Cursor::new(buffer));

    for coordinates in reader.lines().map(|line| {
        line.unwrap()
            .split_whitespace()
            .map(|value| value.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    }) {
        let x = coordinates[0] as usize - origin.0;
        let y = dimension - 1 - (coordinates[1] as usize - origin.1);
        let height = (coordinates[2] / max_height * u16::MAX as f64) as u16;

        data[y * dimension + x] = height;
    }

    Ok(DynamicImage::from(
        ImageBuffer::<Luma<u16>, Vec<u16>>::from_vec(dimension as u32, dimension as u32, data)
            .ok_or(anyhow!("Could not create image from the parsed data."))?,
    ))
}

pub(crate) fn parse_albedo(buffer: Vec<u8>) -> Result<DynamicImage> {
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

    // image.save(path)?;

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

    // image.save(path)?;

    Ok(())
}
