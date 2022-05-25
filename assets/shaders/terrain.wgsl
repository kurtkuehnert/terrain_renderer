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
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] local_position: vec2<f32>;
    [[location(1)]] world_position: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> config: TerrainConfig;
[[group(2), binding(1)]]
var quadtree: texture_2d_array<u32>;
[[group(2), binding(2)]]
var filter_sampler: sampler;
[[group(2), binding(3)]]
var height_atlas: texture_2d_array<f32>;
#ifdef ALBEDO
[[group(2), binding(4)]]
var albedo_atlas: texture_2d_array<f32>;
#endif

// Todo: make these configurable
let height_scale : f32 = 0.96969696969; // 128 / 132
let height_offset: f32 = 0.01515151515; //   2 / 132
let albedo_scale : f32 = 0.9968847352;  // 640 / 642
let albedo_offset: f32 = 0.00155763239; //   1 / 642

[[group(3), binding(0)]]
var<storage> patch_list: PatchList;

// Todo: precompute the node sizes?
fn node_size(lod: u32) -> f32 {
    return f32(config.chunk_size * (1u << lod));
}

#import bevy_terrain::atlas
#import bevy_terrain::debug

fn atlas_lookup(log_distance: f32, world_position: vec2<f32>) -> AtlasLookup {
    let lod = clamp(u32(log_distance), 0u, config.lod_count - 1u);

#ifdef SHOW_NODES
    for (var lod = 0u; lod < config.lod_count; lod = lod + 1u) {
        let coordinate = world_position / node_size(lod);
        let grid_coordinate = floor(view.world_position.xz / node_size(lod) - 0.5 * f32(config.node_count - 1u));

        let grid = step(grid_coordinate, coordinate) * (1.0 - step(grid_coordinate + f32(config.node_count), coordinate));

        if (grid.x * grid.y == 1.0) {
            break;
        }
    }
#endif

    let map_coords = vec2<i32>((world_position / node_size(lod)) % f32(config.node_count));
    let lookup = textureLoad(quadtree, map_coords, i32(lod), 0);

    let atlas_lod = lookup.z;
    let atlas_index =  i32((lookup.x << 8u) + lookup.y);
    let atlas_coords = (world_position / node_size(atlas_lod)) % 1.0;

    return AtlasLookup(atlas_lod, atlas_index, atlas_coords);
}

fn calculate_position(vertex_index: u32, stitch: u32) -> vec2<u32> {
    // use first and last index twice, to form degenerate triangles
    // Todo: documentation
    let row_index = clamp(vertex_index % config.vertices_per_row, 1u, config.vertices_per_row - 2u) - 1u;
    var vertex_position = vec2<u32>((row_index & 1u) + vertex_index / config.vertices_per_row, row_index >> 1u);

    // stitch the edges of the patches together
    if (vertex_position.x == 0u && (stitch & 1u) != 0u) {
        vertex_position.y = vertex_position.y & 0xFFFEu; // mod 2
    }
    if (vertex_position.y == 0u && (stitch & 2u) != 0u) {
        vertex_position.x = vertex_position.x & 0xFFFEu; // mod 2
    }
    if (vertex_position.x == config.patch_size && (stitch & 4u) != 0u) {
        vertex_position.y = vertex_position.y + 1u & 0xFFFEu; // mod 2
    }
    if (vertex_position.y == config.patch_size && (stitch & 8u) != 0u) {
        vertex_position.x = vertex_position.x + 1u & 0xFFFEu; // mod 2
    }

    return vertex_position;
}

fn calculate_normal(uv: vec2<f32>, atlas_index: i32, lod: u32) -> vec3<f32> {
    let left  = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>(-1,  0)).x;
    let up    = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 0, -1)).x;
    let right = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 1,  0)).x;
    let down  = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 0,  1)).x;

    return normalize(vec3<f32>(right - left, f32(2u << lod) / config.height, down - up));
}

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let patch = patch_list.data[vertex.instance];
    let vertex_position = calculate_position(vertex.index, patch.stitch);
    let local_position = vec2<f32>((patch.coords + vertex_position) * patch.size);

    let distance = distance(local_position, view.world_position.xz);
    let log_distance = log2(distance / config.view_distance);
    let lookup = atlas_lookup(log_distance, local_position);
    let atlas_index = lookup.atlas_index;
    let atlas_coords = lookup.atlas_coords;
    let height_coords = atlas_coords * height_scale + height_offset;

    let height = config.height * textureSampleLevel(height_atlas, filter_sampler, height_coords, atlas_index, 0.0).x;
    let world_position = mesh.model * vec4<f32>(local_position.x, height, local_position.y, 1.0);

    var fragment: Fragment;
    fragment.clip_position = view.view_proj * world_position;
    fragment.local_position = vec2<f32>(local_position);
    fragment.world_position = world_position.xyz;
    fragment.color = vec4<f32>(0.0);

#ifdef SHOW_PATCHES
    fragment.color = show_patches(patch, vertex_position);
#endif

    return fragment;
}

fn color_fragment(
    world_position: vec3<f32>,
    lod: u32,
    atlas_index: i32,
    atlas_coords: vec2<f32>
) -> vec4<f32> {
    var color = vec4<f32>(0.0);

    let height_coords = atlas_coords * height_scale + height_offset;
    let albedo_coords = atlas_coords * albedo_scale + albedo_offset;

    #ifndef COLOR
        color = mix(color, vec4<f32>(1.0), 0.5);
    #endif

    #ifdef SHOW_LOD
        color = mix(color, show_lod(lod, world_position.xz), 0.4);
    #endif

    #ifdef ALBEDO
        color = mix(color, textureSample(albedo_atlas, filter_sampler, albedo_coords, atlas_index), 0.5);
    #endif

    #ifdef UV
        color = mix(color, vec4<f32>(atlas_coords.x, atlas_coords.y, 0.0, 1.0), 0.5);
    #endif

    #ifdef LIGHTING
        let ambient = 0.1;
        let light_pos = vec3<f32>(5000.0, 2000.0, 5000.0);
        let direction = normalize(light_pos - world_position);

        let normal = calculate_normal(height_coords, atlas_index, lod);

        let diffuse = max(dot(direction, normal), 0.0);

        color = color * (ambient + diffuse);
    #endif

    return color;
}

[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    let distance = distance(fragment.local_position, view.world_position.xz);
    let log_distance = log2(distance / config.view_distance);

    var output_color = fragment.color;

    let lookup = atlas_lookup(log_distance, fragment.local_position);
    var color = color_fragment(fragment.world_position, lookup.lod, lookup.atlas_index, lookup.atlas_coords);

    let blend = 0.3;
    let ratio = (1.0 - log_distance % 1.0) / blend;

    if (ratio < 1.0) {
        let lookup2 = atlas_lookup(log_distance + 1.0, fragment.local_position);
        let color2 = color_fragment(fragment.world_position, lookup2.lod, lookup2.atlas_index, lookup2.atlas_coords);
        color = mix(color2, color, ratio);
    }

    return mix(fragment.color, color, 0.8);
}
