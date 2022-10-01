#![allow(dead_code)]
#![allow(unused_variables)]

use crate::system_adapter::unwrap;
use crate::In;
use bevy::prelude::UVec2;
use bevy_terrain::prelude::{AttachmentConfig, AttachmentFormat, FileFormat};
use bevy_terrain::preprocess::file_io::{load_image, reset_directory, save_image};
use bevy_terrain::preprocess::Rgb8Image;
use image::{io::Reader, DynamicImage, ImageBuffer, Luma, Rgb};
use itertools::iproduct;
use std::iter::{Filter, Map};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

pub(crate) enum ParseFormat {
    XYZ {
        scale: usize,
        dimension: usize,
        max_height: f32,
    },
    TIF,
}

fn parse_xyz(
    file_path: PathBuf,
    origin_x: usize,
    origin_y: usize,
    scale: usize,
    dimension: usize,
    max_height: f32,
) -> DynamicImage {
    let mut data = vec![0; (dimension * dimension) as usize];

    let file = File::open(file_path).expect("Unable to open file.");
    let reader = BufReader::new(file);

    for line in reader.lines().map(Result::unwrap) {
        let mut coordinate = line.split_whitespace();

        let x = (coordinate
            .next()
            .and_then(|value| value.split_once('.'))
            .and_then(|(value, _)| value.parse::<usize>().ok())
            .unwrap()
            - origin_x)
            / scale;
        let y = dimension
            - 1
            - (coordinate
                .next()
                .and_then(|value| value.split_once('.'))
                .and_then(|(value, _)| value.parse::<usize>().ok())
                .unwrap()
                - origin_y)
                / scale;
        let height = (coordinate.next().unwrap().parse::<f32>().unwrap() / max_height
            * u16::MAX as f32) as u16;

        data[y * dimension + x] = height;
    }

    DynamicImage::from(
        ImageBuffer::<Luma<u16>, Vec<u16>>::from_vec(dimension as u32, dimension as u32, data)
            .unwrap(),
    )
}

fn parse_tif(file_path: PathBuf) -> DynamicImage {
    let image = load_image(file_path.to_str().unwrap(), FileFormat::TIF).unwrap();

    let image = image.as_rgb8().unwrap();

    let size = image.width() / 5;

    let mut downscaled_image = Rgb8Image::new(size, size);

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

    DynamicImage::from(downscaled_image)
}

fn iterate_dir(directory: &str) -> Vec<DirEntry> {
    let walker = WalkDir::new(directory).into_iter();
    walker
        .filter_entry(|entry| {
            if entry.path().is_dir() {
                return true;
            }

            if let Some(extension) = entry.path().extension() {
                if (extension == "tif" || extension == "xyz")
                    && !entry
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with("."))
                        .unwrap_or(false)
                {
                    return true;
                }
            }

            false
        })
        .filter_map(|entry| {
            let entry = entry.unwrap();

            if entry.path().is_dir() {
                None
            } else {
                Some(entry)
            }
        })
        .collect::<Vec<_>>()
}

pub(crate) fn parse(
    input_directory: &str,
    output_directory: &str,
    name: &str,
    format: ParseFormat,
    file_format: FileFormat,
) {
    reset_directory(output_directory);

    let mut count = 0;
    let mut n = 0;

    let mut min_pos = UVec2::splat(u32::MAX);
    let mut max_pos = UVec2::splat(u32::MIN);

    let files = iterate_dir(input_directory);

    for entry in &files {
        let mut parts = entry.file_name().to_str().unwrap().split('.');
        let mut parts = parts.next().unwrap().split('_');
        parts.next();

        let coord = UVec2::new(
            parts.next().unwrap().parse::<u32>().unwrap() % 1000,
            parts.next().unwrap().parse::<u32>().unwrap(),
        );

        min_pos = min_pos.min(coord);
        max_pos = max_pos.max(coord);

        count += 1;
    }

    let pos = UVec2::new(min_pos.x, max_pos.y);
    let size = (max_pos - min_pos) / 2 + 1;
    println!("Pos: {min_pos}");
    println!("Size: {size}");

    for entry in &files {
        let mut parts = entry.file_name().to_str().unwrap().split('.');
        let mut parts = parts.next().unwrap().split('_');
        parts.next();

        let coord = UVec2::new(
            parts.next().unwrap().parse::<u32>().unwrap() % 1000,
            parts.next().unwrap().parse::<u32>().unwrap(),
        );
        let tile_x = (coord.x - pos.x) / 2;
        let tile_y = (pos.y - coord.y) / 2;

        let path = entry.clone().into_path();

        let image = match format {
            ParseFormat::XYZ {
                scale,
                dimension,
                max_height,
            } => parse_xyz(
                path,
                1000 * coord.x as usize,
                1000 * coord.y as usize,
                scale,
                dimension,
                max_height,
            ),
            ParseFormat::TIF => parse_tif(path),
        };

        let path = format!("{output_directory}/{name}_{tile_x}_{tile_y}");

        save_image(
            &path,
            &image,
            &AttachmentConfig {
                name: "".to_string(),
                texture_size: 0,
                center_size: 0,
                border_size: 0,
                mip_level_count: 0,
                format: match format {
                    ParseFormat::XYZ { .. } => AttachmentFormat::R16,
                    ParseFormat::TIF => AttachmentFormat::Rgb8,
                },
                file_format,
            },
        );

        n += 1;
        println!("Finished parsing {n} of {count} files.");
    }
}
