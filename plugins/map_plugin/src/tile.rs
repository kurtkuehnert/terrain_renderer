use crate::bundles::TileBundle;
use crate::data::ClipmapData;
use crate::pipeline::MapMaterial;
use bevy::{
    math::{Vec2, Vec3},
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use bytemuck::cast_vec;
use itertools::iproduct;

pub const TILE_ALIGNMENTS: [[(TileVariant, f32); 4]; 4] = [
    [
        (TileVariant::Corner, 0.0),
        (TileVariant::Edge, 0.0),
        (TileVariant::Edge, 0.0),
        (TileVariant::Corner, 270.0),
    ],
    [
        (TileVariant::Edge, 90.0),
        (TileVariant::Normal, 0.0),
        (TileVariant::Normal, 0.0),
        (TileVariant::Edge, 270.0),
    ],
    [
        (TileVariant::Edge, 90.0),
        (TileVariant::Normal, 0.0),
        (TileVariant::Normal, 0.0),
        (TileVariant::Edge, 270.0),
    ],
    [
        (TileVariant::Corner, 90.0),
        (TileVariant::Edge, 180.0),
        (TileVariant::Edge, 180.0),
        (TileVariant::Corner, 180.0),
    ],
];

#[derive(Default, Component)]
pub struct Tile;

#[derive(Eq, PartialEq)]
pub enum TileVariant {
    Normal,
    Edge,
    Corner,
}

pub struct TileShape {
    wireframe: bool,
    /// Number of quads along the side of a tile.
    side_length: u32,
    /// Number of vertices along the side of a tile.
    vertex_count: u32,
    positions: Vec<Vec3>,
}

impl TileShape {
    #[allow(clippy::ptr_arg)]
    pub fn new(side_length: u32, wireframe: bool) -> Self {
        let vertex_count = side_length + 1;

        let mut positions = Vec::with_capacity((vertex_count * vertex_count) as usize);

        for (x, y) in iproduct!((0..vertex_count), (0..vertex_count)) {
            // position percentage relative the the mesh (0, 0) -> bottom left, (1, 1) -> top right
            let percent = Vec2::new(x as f32, y as f32) / side_length as f32;

            // calculate the vertex position, while mapping the center to (0,0)
            let position = Vec3::new(percent.x - 0.5, 0.0, percent.y - 0.5) * side_length as f32;

            positions.push(position);
        }

        Self {
            wireframe,
            side_length,
            vertex_count,
            positions,
        }
    }

    pub fn to_mesh(&self, variant: TileVariant) -> Mesh {
        let indices = self.calculate_indices(variant);

        let mut mesh = Mesh::new(if self.wireframe {
            PrimitiveTopology::LineList
        } else {
            PrimitiveTopology::TriangleList
        });

        // set the attributes of the mesh
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            cast_vec::<Vec3, [f32; 3]>(self.positions.clone()),
        );

        mesh
    }

    fn calculate_indices(&self, variant: TileVariant) -> Vec<u32> {
        let mut indices = Vec::with_capacity((self.side_length * self.side_length * 6) as usize);

        // a---b---c
        // |   |   |
        // d---e---f
        // |   |   |
        // g---h---i
        for (x, y) in iproduct!(
            (0_u32..self.side_length).step_by(2),
            (0_u32..self.side_length).step_by(2)
        ) {
            let a = y * self.vertex_count + x;
            let b = a + 1;
            let c = a + 2;
            let d = a + self.vertex_count;
            let e = a + self.vertex_count + 1;
            let f = a + self.vertex_count + 2;
            let g = a + 2 * self.vertex_count;
            let h = a + 2 * self.vertex_count + 1;
            let i = a + 2 * self.vertex_count + 2;

            if x == 0 && variant != TileVariant::Normal {
                self.add_triangle(&mut indices, e, g, a);
            } else {
                self.add_triangle(&mut indices, e, g, d);
                self.add_triangle(&mut indices, e, d, a);
            }

            if y == 0 && variant == TileVariant::Corner {
                self.add_triangle(&mut indices, e, a, c);
            } else {
                self.add_triangle(&mut indices, e, a, b);
                self.add_triangle(&mut indices, e, b, c);
            }

            self.add_triangle(&mut indices, e, c, f);
            self.add_triangle(&mut indices, e, f, i);
            self.add_triangle(&mut indices, e, i, h);
            self.add_triangle(&mut indices, e, h, g);
        }

        indices
    }

    fn add_triangle(&self, indices: &mut Vec<u32>, a: u32, b: u32, c: u32) {
        if self.wireframe {
            indices.push(a);
            indices.push(b);
            indices.push(b);
            indices.push(c);
            indices.push(c);
            indices.push(a);
        } else {
            indices.push(a);
            indices.push(b);
            indices.push(c);
        }
    }
}

pub fn spawn_clipmap_tiles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    clipmap_data: &ClipmapData,
    map_material: Handle<MapMaterial>,
) -> Vec<Entity> {
    let tile = TileShape::new(clipmap_data.side_length, clipmap_data.wireframe);
    let normal_tile = meshes.add(tile.to_mesh(TileVariant::Normal));
    let edge_tile = meshes.add(tile.to_mesh(TileVariant::Edge));
    let corner_tile = meshes.add(tile.to_mesh(TileVariant::Corner));

    let mut tiles = Vec::new();

    for level in (0..clipmap_data.lod_levels).map(|level| 2_i32.pow(level) as f32) {
        for (x, y) in iproduct!((0..4), (0..4)) {
            if level == 1.0 || x == 0 || x == 3 || y == 0 || y == 3 {
                let side_length = clipmap_data.side_length as f32 * level;

                let (variant, angle) = &TILE_ALIGNMENTS[y][x];

                let mesh = match variant {
                    TileVariant::Normal => normal_tile.clone(),
                    TileVariant::Edge => edge_tile.clone(),
                    TileVariant::Corner => corner_tile.clone(),
                };

                let translation = Vec3::new(
                    (x as f32 - 1.5) * side_length,
                    level * 2.0,
                    (y as f32 - 1.5) * side_length,
                );
                let rotation = Quat::from_rotation_y(angle.to_radians());
                let scale = Vec3::ONE * level;

                tiles.push(
                    commands
                        .spawn_bundle(TileBundle {
                            mesh,
                            map_material: map_material.clone(),
                            transform: Transform {
                                translation,
                                rotation,
                                scale,
                            },
                            ..Default::default()
                        })
                        .insert(Name::new(format!("Tile: level={} x={} y={}", level, x, y)))
                        .id(),
                );
            }
        }
    }

    tiles
}
