use bevy::{
    math::{Vec2, Vec3},
    prelude::Mesh,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use itertools::iproduct;

pub struct Cubesphere {
    pub radius: f32,
    pub grid_size: usize,
}

impl Default for Cubesphere {
    fn default() -> Self {
        Self {
            radius: 1.0,
            grid_size: 5,
        }
    }
}

impl From<Cubesphere> for Mesh {
    fn from(sphere: Cubesphere) -> Self {
        let side_vertex_count = sphere.grid_size + 1;
        let half_grid_size = sphere.grid_size as f32 / 2.0;
        let last = sphere.grid_size;

        let index_count = sphere.grid_size * sphere.grid_size * 36;
        let vertex_count = side_vertex_count * side_vertex_count * 6;

        let mut indices: Vec<u32> = Vec::with_capacity(index_count);
        let mut points: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);

        let iterator =
            iproduct!((0..6), (0..side_vertex_count), (0..side_vertex_count)).enumerate();

        for (index, (side, x, y)) in iterator {
            let u = x as f32 / half_grid_size - 1.0;
            let v = y as f32 / half_grid_size - 1.0;

            // matches the uv of the bevy cube
            let (cube_point, cw) = match side {
                0 => (Vec3::new(-u, -v, -1.0), true),  // right
                1 => (Vec3::new(u, v, 1.0), false),    // left
                2 => (Vec3::new(-u, -1.0, -v), false), // bottom
                3 => (Vec3::new(u, 1.0, v), true),     // top
                4 => (Vec3::new(-1.0, -u, -v), true),  // back
                5 => (Vec3::new(1.0, u, v), false),    // front
                _ => unreachable!(),
            };

            let x2 = cube_point.x * cube_point.x;
            let y2 = cube_point.y * cube_point.y;
            let z2 = cube_point.z * cube_point.z;

            // use the mapping described here: https://catlikecoding.com/unity/tutorials/cube-sphere/
            // instead of simple vector normalization to reduce distortion
            let normal = Vec3::new(
                (1.0 - y2 / 2.0 - z2 / 2.0 + y2 * z2 / 3.0).sqrt(),
                (1.0 - x2 / 2.0 - z2 / 2.0 + x2 * z2 / 3.0).sqrt(),
                (1.0 - x2 / 2.0 - y2 / 2.0 + x2 * y2 / 3.0).sqrt(),
            ) * cube_point;

            let point = normal * sphere.radius;

            let uv = Vec2::new((u + 1.0) / 2.0, (v + 1.0) / 2.0);

            points.push(point.into());
            normals.push(normal.into());
            uvs.push(uv.into());

            if x < last && y < last {
                // determine the remaining three vertex indices
                let index_a = index as u32;
                let index_b = index_a + 1;
                let index_c = index_a + side_vertex_count as u32 + 1;
                let index_d = index_a + side_vertex_count as u32;

                if cw {
                    indices.push(index_a);
                    indices.push(index_b);
                    indices.push(index_c);
                    indices.push(index_a);
                    indices.push(index_c);
                    indices.push(index_d);
                } else {
                    indices.push(index_a);
                    indices.push(index_c);
                    indices.push(index_b);
                    indices.push(index_a);
                    indices.push(index_d);
                    indices.push(index_c);
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
