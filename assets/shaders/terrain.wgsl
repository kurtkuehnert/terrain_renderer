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
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
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
[[group(2), binding(4)]]
var albedo_atlas: texture_2d_array<f32>;

[[group(3), binding(0)]]
var<storage> patch_list: PatchList;

#import bevy_terrain::atlas

fn atlas_lookup(world_position: vec2<f32>) -> AtlasLookup {
    let dist = view.world_position.xyz - vec3<f32>(world_position.x, 500.0, world_position.y);

    let layer = clamp(u32(sqrt(length(dist)) / 30.0), 0u, config.lod_count - 1u);
    let layer = 0u;

    let map_coords =  vec2<i32>(world_position / f32(config.chunk_size * (1u << layer)));
    let lookup = textureLoad(quadtree, map_coords, i32(layer), 0);

    let lod = lookup.z;
    let atlas_index =  i32((lookup.x << 8u) + lookup.y);
    let atlas_coords = (world_position / f32(config.chunk_size * (1u << lod))) % 1.0;

    return AtlasLookup(lod, atlas_index, atlas_coords);
}

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

fn calculate_normal(uv: vec2<f32>, atlas_index: i32, lod: u32) -> vec3<f32> {
    let left  = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>(-1,  0)).x;
    let up    = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 0, -1)).x;
    let right = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 1,  0)).x;
    let down  = textureSampleLevel(height_atlas, filter_sampler, uv, atlas_index, 0.0, vec2<i32>( 0,  1)).x;
    let normal = normalize(vec3<f32>(right - left, 2.0 * f32(1u << lod) / config.height, down - up));

    return normal;
}

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let patch = patch_list.data[vertex.instance];

    let vertex_position = calculate_position(vertex.index, patch.lod_delta);

    let world_position = vec2<f32>(
        f32(patch.position.x + vertex_position.x * patch.scale),
        f32(patch.position.y + vertex_position.y * patch.scale),
    );

    let lookup = atlas_lookup(world_position);
    let lod = lookup.lod;
    let atlas_index = lookup.atlas_index;
    let atlas_coords = lookup.atlas_coords;

    var height = textureSampleLevel(height_atlas, filter_sampler, atlas_coords, atlas_index, 0.0).r;

    // discard vertecies with height 0
    if (height == 0.0) {
        height = height / 0.0;
    }

    height = config.height * height;
    let world_position = mesh.model * vec4<f32>(world_position.x, height, world_position.y, 1.0);

    var out: Fragment;
    out.position = view.view_proj * world_position;
    out.world_position = world_position;

    let colored = true;
    // let colored = false;

    if (colored) {
        if (lod == 0u) {
            out.color = vec4<f32>(1.0,0.0,0.0,1.0);
        }
        if (lod == 1u) {
            out.color = vec4<f32>(0.0,1.0,0.0,1.0);
        }
        if (lod == 2u) {
            out.color = vec4<f32>(0.0,0.0,1.0,1.0);
        }
        if (lod == 3u) {
            out.color = vec4<f32>(1.0,1.0,0.0,1.0);
        }
        if (lod == 4u) {
            out.color = vec4<f32>(1.0,0.0,1.0,1.0);
        }
        // if (patch.lod_delta != 0u) {
        //     out.color = vec4<f32>(0.0,1.0,1.0,1.0);
        // }
    }
    else {
        out.color = vec4<f32>(1.0);
    }

    if (false) {
        if ((patch.position.x + patch.position.y * 8u) % 3u == 0u) {
            out.color = vec4<f32>(1.0,0.0,0.0,1.0);
        }
        if ((patch.position.x + patch.position.y * 8u) % 3u == 1u) {
            out.color = vec4<f32>(0.0,1.0,0.0,1.0);
        }
        if ((patch.position.x + patch.position.y * 8u) % 3u == 2u) {
            out.color = vec4<f32>(0.0,0.0,1.0,1.0);
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

    let lookup = atlas_lookup(fragment.world_position.xz);
    let lod = lookup.lod;
    let atlas_index = lookup.atlas_index;
    let atlas_coords = lookup.atlas_coords;

    let albedo = true;
    let albedo = false;

    if (albedo) {
        output_color = output_color * 0.1 + textureSample(albedo_atlas, filter_sampler, atlas_coords, atlas_index);
    }

    let lighting = true;
    // let lighting = false;

    if (lighting) {
        let ambient = 0.1;
        let light_pos = vec3<f32>(5000.0, 2000.0, 5000.0);
        let direction = normalize(light_pos - fragment.world_position.xyz);
        let normal = calculate_normal(atlas_coords, atlas_index, lod);

        let diffuse = max(dot(direction, normal), 0.0);

        output_color = output_color * (ambient + diffuse) * 1.0;
    }

    return output_color;
}
