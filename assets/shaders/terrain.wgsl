// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct
#import bevy_terrain::config
#import bevy_terrain::patch

// vertex intput
struct Vertex {
    [[builtin(instance_index)]] instance: u32;
    [[builtin(vertex_index)]] index: u32;
};

// fragment input
struct Fragment {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] world_position: vec4<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] atlas_index: i32;
    [[location(4)]] scale: f32;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> config: TerrainConfig;
[[group(2), binding(2)]]
var height_atlas: texture_2d_array<f32>;
[[group(2), binding(3)]]
var height_sampler: sampler;

[[group(3), binding(0)]]
var<storage> patch_list: PatchList;

fn calculate_position(vertex_index: u32, lod_delta: u32) -> vec2<u32> {
    // use first and last index twice, to form degenerate triangles
    let row_index = clamp(vertex_index % config.vertices_per_row, 1u, config.vertices_per_row - 2u) - 1u;
    var vertex_position = vec2<u32>((row_index & 1u) + vertex_index / config.vertices_per_row, row_index >> 1u);

    // stitch the edges of the patches together
    if (vertex_position.x == 0u) {
        let delta_left = lod_delta >> 12u;
        let offset = vertex_position.y % (1u << delta_left);
        vertex_position.y = vertex_position.y - offset;
    }
    if (vertex_position.y == 0u) {
        let delta_up = (lod_delta >> 8u) & 7u;
        let offset = vertex_position.x % (1u << delta_up);
        vertex_position.x = vertex_position.x - offset;
    }
    if (vertex_position.x == config.patch_size) {
        let delta_right = (lod_delta >> 4u) & 7u;
        let offset = vertex_position.y % (1u << delta_right);
        vertex_position.y = vertex_position.y - offset;
    }
    if (vertex_position.y == config.patch_size) {
        let delta_down = lod_delta & 7u;
        let offset = vertex_position.x % (1u << delta_down);
        vertex_position.x = vertex_position.x - offset;
    }

    return vertex_position;
}

fn calculate_normal(uv: vec2<f32>, atlas_index: i32, scale: f32) -> vec3<f32> {
    let left  = config.height * textureSample(height_atlas, height_sampler, uv, atlas_index, vec2<i32>(-1,  0)).x;
    let up    = config.height * textureSample(height_atlas, height_sampler, uv, atlas_index, vec2<i32>( 0, -1)).x;
    let right = config.height * textureSample(height_atlas, height_sampler, uv, atlas_index, vec2<i32>( 1,  0)).x;
    let down  = config.height * textureSample(height_atlas, height_sampler, uv, atlas_index, vec2<i32>( 0,  1)).x;
    let normal = normalize(vec3<f32>(right - left, 2.0 * scale, down - up));

    return normal;
}

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let patch = patch_list.data[vertex.instance];

    let vertex_position = calculate_position(vertex.index, patch.lod_delta);

    let coords = vec2<i32>(
        i32(vertex_position.x + config.patch_size * (patch.coord_offset & 7u)),
        i32(vertex_position.y + config.patch_size * (patch.coord_offset >> 3u))
    );

    let height = textureLoad(height_atlas, coords, i32(patch.atlas_index), 0).r;

    let world_position = mesh.model * vec4<f32>(
        f32(patch.position.x + vertex_position.x * patch.scale),
        config.height * height,
        f32(patch.position.y + vertex_position.y * patch.scale),
        1.0
    );

    var out: Fragment;
    out.position = view.view_proj * world_position;
    out.world_position = world_position;
    out.uv = vec2<f32>(coords) / f32(config.texture_size);
    out.atlas_index = i32(patch.atlas_index);
    out.scale = f32(patch.scale);
    out.color = vec4<f32>(1.0);

    let colored = true;
    let colored = false;

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

    return out;
}

[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    var output_color = fragment.color;

    let ambient = 0.1;
    let light_pos = vec3<f32>(5000.0);
    let direction = normalize(light_pos - fragment.world_position.xyz);
    let normal = calculate_normal(fragment.uv, fragment.atlas_index, fragment.scale);
    let diffuse = max(dot(direction, normal), 0.0);

    output_color = output_color * (ambient + diffuse) * 0.5;

    return output_color;
}
