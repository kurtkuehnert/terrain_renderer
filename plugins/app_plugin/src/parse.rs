use image::{imageops, io::Reader, GenericImage, ImageBuffer, Luma, RgbaImage};

use itertools::iproduct;
use std::fs;

fn parse_height(value: &str) -> u16 {
    let max_height = 1000.0;

    let height: f32 = value.parse().expect("Could not parse value of file.");
    (height / max_height * u16::MAX as f32) as u16
}

fn parse_xyz(path: &str) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let dimension: u32 = 2000;

    let data = fs::read_to_string(path).expect("Unable to open file.");
    let data = data
        .split_whitespace()
        .skip(2)
        .step_by(3)
        .map(parse_height)
        .collect::<Vec<_>>();

    ImageBuffer::from_vec(dimension, dimension, data).unwrap()
}

pub fn process_height(path: &str, size: u32) {
    let dimension = 2000;
    let mut output = <ImageBuffer<Luma<u16>, _>>::new(size * dimension, size * dimension);

    for (x, y) in iproduct!(0..size, 0..size) {
        let coord_x = 33334 + x * 2;
        let coord_y = 5614 + y * 2;

        let path =
            format!("data/dgm1_{coord_x}_{coord_y}_2_sn_xyz/dgm1_{coord_x}_{coord_y}_2_sn.xyz");
        let input = parse_xyz(&path);

        output
            .copy_from(&input, dimension * x, dimension * y)
            .unwrap();
    }

    imageops::flip_horizontal_in_place(&mut output);

    output.save(path).unwrap();
}

pub fn process_albedo(path: &str, size: u32) {
    let dimension = 10000;
    let mut output = RgbaImage::new(dimension * size, dimension * size);

    for (x, y) in iproduct!(0..size, 0..size) {
        let coord_x = 33334 + x * 2;
        let coord_y = 5614 + y * 2;

        let path = format!(
            "data/dop20rgb_{coord_x}_{coord_y}_2_sn_tiff/dop20rgb_{coord_x}_{coord_y}_2_sn.png"
        );
        let mut reader = Reader::open(path).unwrap();
        reader.no_limits();
        let input = reader.decode().unwrap();

        output
            .copy_from(&input, dimension * x, dimension * (1 - y))
            .unwrap();
    }

    imageops::rotate180(&mut output);

    output.save(path).unwrap();
}
