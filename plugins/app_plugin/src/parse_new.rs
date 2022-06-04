#![allow(dead_code)]
#![allow(unused_variables)]

use image::io::Reader;
use image::{imageops, imageops::FilterType, GenericImage, ImageBuffer, Luma};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

const MAX_HEIGHT: f32 = 1000.0;
const DGM01_SCALE: usize = 1;
const DGM01_DIMENSION: usize = 2000 / DGM01_SCALE;
const DGM01_TILE_SIZE: u32 = 8 * DGM01_DIMENSION as u32;
const DGM20_SCALE: usize = 20;
const DGM20_DIMENSION: usize = 2000 / DGM20_SCALE;
const DGM20_TILE_SIZE: u32 = 128 * DGM20_DIMENSION as u32;
const DGM16_TILE_SIZE: u32 = DGM20_TILE_SIZE * 20 / 16;

fn parse_xyz(
    file_path: PathBuf,
    origin_x: usize,
    origin_y: usize,
    dimension: usize,
    scale: usize,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
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
        let height = (coordinate.next().unwrap().parse::<f32>().unwrap() / MAX_HEIGHT
            * u16::MAX as f32) as u16;

        data[y * dimension + x] = height;
    }

    ImageBuffer::from_vec(dimension as u32, dimension as u32, data).unwrap()
}

pub(crate) fn parse_dgm01(input_directory: &str, output_directory: &str) {
    fs::remove_dir_all(output_directory).unwrap();
    fs::create_dir(output_directory).unwrap();

    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");
    let count = paths.count() - 1;
    let mut n = 1;
    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");

    for path in paths {
        let dir_path = path.unwrap().path();
        if !dir_path.is_dir() {
            continue;
        }

        let paths = fs::read_dir(dir_path).expect("Could not find the input directory.");
        for path in paths {
            let file_path = path.unwrap().path();

            if file_path.extension().unwrap() != "xyz" {
                continue;
            }

            let file_name = file_path
                .with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let origin_x = file_name[7..10].parse::<usize>().unwrap();
            let origin_y = file_name[11..15].parse::<usize>().unwrap();

            let initial_pos = (328, 5620);

            let tile_x = (origin_x - initial_pos.0) / 2;
            let tile_y = (initial_pos.1 - origin_y) / 2;

            let image = parse_xyz(
                file_path,
                origin_x * 1000,
                origin_y * 1000,
                DGM01_DIMENSION,
                DGM01_SCALE,
            );

            image
                .save(format!("{output_directory}/dgm01_{tile_x}_{tile_y}.png"))
                .expect("Could not save file.");
        }

        println!("Finished parsing {n} of {count} files.");
        n += 1;
    }
}

pub(crate) fn parse_dop20(input_directory: &str, output_directory: &str) {
    fs::remove_dir_all(output_directory).unwrap();
    fs::create_dir(output_directory).unwrap();

    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");
    let count = paths.count() - 1;
    let mut n = 1;
    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");

    for path in paths {
        let dir_path = path.unwrap().path();
        if !dir_path.is_dir() {
            continue;
        }

        let paths = fs::read_dir(dir_path).expect("Could not find the input directory.");
        for path in paths {
            let file_path = path.unwrap().path();

            if file_path.extension().unwrap() != "tif" {
                continue;
            }

            let file_name = file_path
                .with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let origin_x = file_name[11..14].parse::<usize>().unwrap();
            let origin_y = file_name[15..19].parse::<usize>().unwrap();

            let initial_pos = (328, 5620);

            let tile_x = (origin_x - initial_pos.0) / 2;
            let tile_y = (initial_pos.1 - origin_y) / 2;

            let mut reader = Reader::open(file_path).unwrap();
            reader.no_limits();
            let mut image = reader.decode().unwrap();

            image
                .save(format!("{output_directory}/dop20_{tile_x}_{tile_y}.png"))
                .expect("Could not save file.");
        }

        println!("Finished parsing {n} of {count} files.");
        n += 1;
    }
}

pub(crate) fn parse_dgm20(input_directory: &str, output_directory: &str) {
    fs::remove_dir_all(output_directory).unwrap();
    fs::create_dir(output_directory).unwrap();

    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");

    for path in paths {
        let file_path = path.unwrap().path();
        let file_name = file_path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let origin_x = file_name[8..11].parse::<usize>().unwrap();
        let origin_y = file_name[12..16].parse::<usize>().unwrap();

        let initial_pos = (278, 5780);

        let tile_x = (origin_x - initial_pos.0) / 2;
        let tile_y = (initial_pos.1 - origin_y) / 2;

        let image = parse_xyz(
            file_path,
            origin_x * 1000,
            origin_y * 1000,
            DGM20_DIMENSION,
            DGM20_SCALE,
        );

        image
            .save(format!("{output_directory}/dgm20_{tile_x}_{tile_y}.png"))
            .expect("Could not save file.");
    }
}

pub(crate) fn combine_dgm20_as_dgm16(input_directory: &str, output_file_path: &str) {
    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");

    let mut output = <ImageBuffer<Luma<u16>, _>>::new(DGM20_TILE_SIZE, DGM20_TILE_SIZE);

    for path in paths {
        let file_path = path.unwrap().path();
        let file_name = file_path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let initial_pos = (278, 5560);

        let x = (&file_name[8..11].parse::<usize>().unwrap() - initial_pos.0) / 2;
        let y = (&file_name[12..16].parse::<usize>().unwrap() - initial_pos.1) / 2;

        let input = image::open(file_path).unwrap().into_luma16();

        output
            .copy_from(
                &input,
                (DGM20_DIMENSION * x) as u32,
                (DGM20_DIMENSION * y) as u32,
            )
            .unwrap();
    }

    let mut output = imageops::resize(
        &mut output,
        DGM16_TILE_SIZE,
        DGM16_TILE_SIZE,
        FilterType::Lanczos3,
    );
    imageops::flip_horizontal_in_place(&mut output);

    output.save(output_file_path).expect("Could not save file.");
}

pub(crate) fn combine_dgm01(input_directory: &str, output_file_path: &str) {
    let paths = fs::read_dir(input_directory).expect("Could not find the input directory.");

    let mut output = <ImageBuffer<Luma<u16>, _>>::new(DGM01_TILE_SIZE, DGM01_TILE_SIZE);

    for path in paths {
        let file_path = path.unwrap().path();
        let file_name = file_path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let initial_pos = (328, 5606);

        let x = (&file_name[7..10].parse::<usize>().unwrap() - initial_pos.0) / 2;
        let y = (&file_name[11..15].parse::<usize>().unwrap() - initial_pos.1) / 2;

        let input = image::open(file_path).unwrap().into_luma16();

        output
            .copy_from(
                &input,
                (DGM01_DIMENSION * x) as u32,
                (DGM01_DIMENSION * y) as u32,
            )
            .unwrap();
    }

    imageops::flip_horizontal_in_place(&mut output);

    output.save(output_file_path).expect("Could not save file.");
}
