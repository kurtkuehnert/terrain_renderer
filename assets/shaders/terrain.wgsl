// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct TerrainConfig {
    lod_count: u32;
    chunk_size: u32;
    patch_size: u32;
    vertices_per_row: u32;
    area_count: vec2<u32>;
    scale: f32;
    height: f32;
};

struct PatchInfo {
    position: vec2<u32>;
    scale: u32;
    atlas_index: u32;
    coord_offset: u32;
    lod: u32;
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
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] world_position: vec4<f32>;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> config: TerrainConfig;
[[group(2), binding(1)]]
var height_atlas: texture_2d_array<u32>;

[[group(3), binding(0)]]
var<storage> patch_list: PatchList;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let row_index = clamp(vertex.index % config.vertices_per_row, 1u, config.vertices_per_row - 2u) - 1u; // use first and last index twice, to form degenerate triangles
    let vertex_position = vec2<u32>((row_index & 1u) + vertex.index / config.vertices_per_row, row_index >> 1u);

    let patch = patch_list.data[vertex.instance];

    let coords = vec2<i32>(
        i32(vertex_position.x + config.patch_size * (patch.coord_offset & 7u)),
        i32(vertex_position.y + config.patch_size * (patch.coord_offset >> 3u))
    );

    let height = config.height * f32(textureLoad(height_atlas, coords, i32(patch.atlas_index), 0).r) / 65535.0;

    let world_position = mesh.model * vec4<f32>(
        f32(patch.position.x + vertex_position.x * patch.scale),
        height,
        f32(patch.position.y + vertex_position.y * patch.scale),
        1.0
    );

    var out: Fragment;
    out.position = view.view_proj * world_position;
    out.world_position = world_position;

    out.color = vec4<f32>(1.0);

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

    return out;
}

[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    var output_color = fragment.color;
    output_color = output_color / pow(length(view.world_position.xyz - fragment.world_position.xyz), 1.5) * 2000.0;
    return output_color;
}
