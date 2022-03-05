// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct TerrainConfig {
    lod_count: u32;
    patch_size: u32;
    chunk_size: u32;
    chunk_count: vec2<u32>;
    texture_size: u32;
    area_size: u32;
    area_count: vec2<u32>;
    terrain_size: vec2<u32>;
    vertices_per_row: u32;
    scale: f32;
    height: f32;
    node_atlas_size: u32;
};

struct PatchInfo {
    position: vec2<u32>;
    scale: u32;
    atlas_index: u32;
    coord_offset: u32;
    lod: u32;
    lod_delta: u32; // should be u16
};

struct PatchList {
    data: array<PatchInfo>;
};

// vertex intput
struct Vertex {
    [[builtin(instance_index)]] instance: u32;
    [[builtin(vertex_index)]] index: u32;
};

// fragment input
struct Fragment {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] world_position: vec4<f32>;
    [[location(3)]] uv: vec2<f32>;
    [[location(4)]] atlas_index: i32;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> config: TerrainConfig;
[[group(2), binding(2)]]
var height_atlas: texture_2d_array<u32>;
[[group(2), binding(3)]]
var height_sampler: sampler;

[[group(3), binding(0)]]
var<storage> patch_list: PatchList;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let row_index = clamp(vertex.index % config.vertices_per_row, 1u, config.vertices_per_row - 2u) - 1u; // use first and last index twice, to form degenerate triangles
    var vertex_position = vec2<u32>((row_index & 1u) + vertex.index / config.vertices_per_row, row_index >> 1u);

    let patch = patch_list.data[vertex.instance];

    // stitch the edges of the patches together
    if (vertex_position.x == 0u) {
        let delta_left = patch.lod_delta >> 12u;
        let offset = vertex_position.y % (1u << delta_left);
        vertex_position.y = vertex_position.y - offset;
    }
    if (vertex_position.y == 0u) {
        let delta_up = (patch.lod_delta >> 8u) & 7u;
        let offset = vertex_position.x % (1u << delta_up);
        vertex_position.x = vertex_position.x - offset;
    }
    if (vertex_position.x == config.patch_size) {
        let delta_right = (patch.lod_delta >> 4u) & 7u;
        let offset = vertex_position.y % (1u << delta_right);
        vertex_position.y = vertex_position.y - offset;
    }
    if (vertex_position.y == config.patch_size) {
        let delta_down = patch.lod_delta & 7u;
        let offset = vertex_position.x % (1u << delta_down);
        vertex_position.x = vertex_position.x - offset;
    }

    let coords = vec2<i32>(
        i32(vertex_position.x + config.patch_size * (patch.coord_offset & 7u)),
        i32(vertex_position.y + config.patch_size * (patch.coord_offset >> 3u))
    );

    let height = f32(textureLoad(height_atlas, coords, i32(patch.atlas_index), 0).r) / 65535.0;

    let world_position = mesh.model * vec4<f32>(
        f32(patch.position.x + vertex_position.x * patch.scale),
        config.height * height,
        f32(patch.position.y + vertex_position.y * patch.scale),
        1.0
    );

    //     10
    //     |
    // 01--11--21
    //     |
    //     12
    let s01 = config.height * f32(textureLoad(height_atlas, coords + vec2<i32>(-1, 0), i32(patch.atlas_index), 0).r) / 65535.0;
    let s21 = config.height * f32(textureLoad(height_atlas, coords + vec2<i32>(1, 0), i32(patch.atlas_index), 0).r) / 65535.0;
    let s10 = config.height * f32(textureLoad(height_atlas, coords + vec2<i32>(0, -1), i32(patch.atlas_index), 0).r) / 65535.0;
    let s12 = config.height * f32(textureLoad(height_atlas, coords + vec2<i32>(0, 1), i32(patch.atlas_index), 0).r) / 65535.0;
    let normal = normalize(vec3<f32>(s21 - s01, 2.0 * f32(patch.scale), s12 - s10));


    var out: Fragment;
    out.position = view.view_proj * world_position;
    out.world_position = world_position;
    out.normal = normal;
    out.uv = vec2<f32>(coords) / f32(patch.scale * config.patch_size);
    out.atlas_index = i32(patch.atlas_index);

    out.color = vec4<f32>(1.0);

    let colored = true;
    let colored = false;
    let flat = false;
    // let flat = true;

    if (flat) {
        out.normal = vec3<f32>(0.0, 1.0, 0.0);
    }

    if (colored) {
        if (patch.lod == 0u) {
            out.color = vec4<f32>(1.0,0.0,0.0,1.0);
        }
        if (patch.lod == 1u) {
            out.color = vec4<f32>(0.0,1.0,0.0,1.0);
        }
        if (patch.lod == 2u) {
            out.color = vec4<f32>(0.0,0.0,1.0,1.0);
        }
        if (patch.lod == 3u) {
            out.color = vec4<f32>(1.0,1.0,0.0,1.0);
        }
        if (patch.lod == 4u) {
            out.color = vec4<f32>(1.0,0.0,1.0,1.0);
        }
        if (patch.lod_delta != 0u) {
            out.color = vec4<f32>(0.0,1.0,1.0,1.0);
        }
    }

    out.color = out.color * 0.5;

    // out.color = out.color * (height + 0.1);

    return out;
}

[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    var output_color = fragment.color;

    // let s01 = config.height * f32(textureSample(height_atlas, height_sampler, fragment.uv, fragment.atlas_index, vec2<i32>(-1, 0)).r)  / 65535.0;
    // let s21 = config.height * f32(textureSample(height_atlas, height_sampler, fragment.uv, fragment.atlas_index, vec2<i32>(1, 0)).r)  / 65535.0;
    // let s10 = config.height * f32(textureSample(height_atlas, height_sampler, fragment.uv, fragment.atlas_index, vec2<i32>(0, -1)).r)  / 65535.0;
    // let s12 = config.height * f32(textureSample(height_atlas, height_sampler, fragment.uv, fragment.atlas_index, vec2<i32>(0, 1)).r)  / 65535.0;
    // let normal = normalize(vec3<f32>(s21 - s01, 2.0, s12 - s10) / 2.0);

    let ambient = 0.1;
    let light_pos = vec3<f32>(5000.0, 5000.0, 0.0);
    let direction = normalize(light_pos - fragment.world_position.xyz);
    let diffuse = max(dot(direction, fragment.normal), 0.0);

    // output_color = output_color / pow(length(view.world_position.xyz - fragment.world_position.xyz), 1.5) * 2000.0;

    output_color = output_color * (ambient + diffuse);

    return output_color;
}
