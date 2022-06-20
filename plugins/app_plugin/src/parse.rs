#![allow(dead_code)]
#![allow(unused_variables)]

use image::{io::Reader, DynamicImage, ImageBuffer, Luma};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

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

pub(crate) fn parse(
    input_directory: &str,
    output_directory: &str,
    initial_pos: (usize, usize),
    name: &str,
    format: ParseFormat,
) {
    fs::remove_dir_all(output_directory).unwrap();
    fs::create_dir(output_directory).unwrap();

    let count = fs::read_dir(input_directory).unwrap().count() - 1;
    let mut n = 0;

    for path in fs::read_dir(input_directory)
        .unwrap()
        .map(|path| path.unwrap().path())
    {
        let file_path = if path.is_dir() {
            fs::read_dir(path)
                .unwrap()
                .filter_map(|path| {
                    let path = path.unwrap().path();
                    let extension = path.extension().unwrap();

                    if extension == "tif" || extension == "xyz" {
                        Some(path)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap()
        } else {
            if path.file_name().unwrap() == ".DS_Store" {
                continue;
            }

            path
        };

        let file_name = file_path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut parts = file_name.split('_');
        parts.next();

        let origin_x = parts.next().unwrap().parse::<usize>().unwrap() % 1000;
        let origin_y = parts.next().unwrap().parse::<usize>().unwrap();

        let tile_x = (origin_x - initial_pos.0) / 2;
        let tile_y = (initial_pos.1 - origin_y) / 2;

        let image = match format {
            ParseFormat::XYZ {
                scale,
                dimension,
                max_height,
            } => parse_xyz(
                file_path,
                origin_x * 1000,
                origin_y * 1000,
                scale,
                dimension,
                max_height,
            ),
            ParseFormat::TIF => {
                let mut reader = Reader::open(file_path).unwrap();
                reader.no_limits();
                reader.decode().unwrap()
            }
        };

        image
            .save(format!("{output_directory}/{name}_{tile_x}_{tile_y}.png"))
            .expect("Could not save file.");

        n += 1;
        println!("Finished parsing {n} of {count} files.");
    }
}
