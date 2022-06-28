#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types
#import bevy_terrain::config
#import bevy_terrain::patch

// Todo: make these configurable
let height_scale :  f32 = 0.96969696969; // 128 / 132
let height_offset:  f32 = 0.01515151515; //   2 / 132
let albedo_scale :  f32 = 0.9968847352;  // 640 / 642
let albedo_offset:  f32 = 0.00155763239; //   1 / 642
let morph_blend:    f32 = 0.2;
let vertex_blend:   f32 = 0.3;
let fragment_blend: f32 = 0.3;

struct Material {
    flags: u32;
};

let material: Material = Material(0u);

// vertex intput
struct VertexInput {
    [[builtin(instance_index)]] instance: u32;
    [[builtin(vertex_index)]] index: u32;
};

// fragment input
struct FragmentInput {
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] local_position: vec2<f32>;
    [[location(1)]] world_position: vec4<f32>;
    [[location(2)]] color: vec4<f32>;
};

// terrain view bindings
[[group(1), binding(0)]]
var<uniform> view_config: TerrainViewConfig;
[[group(1), binding(1)]]
var quadtree: texture_2d_array<u32>;
[[group(1), binding(2)]]
var<storage> patches: PatchList;

// terrain bindings
[[group(2), binding(0)]]
var<uniform> config: TerrainConfig;
[[group(2), binding(1)]]
var filter_sampler: sampler;
[[group(2), binding(2)]]
var height_atlas: texture_2d_array<f32>;
#ifdef ALBEDO
[[group(2), binding(3)]]
var albedo_atlas: texture_2d_array<f32>;
#endif

// mesh bindings
[[group(3), binding(0)]]
var<uniform> mesh: Mesh;


#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

#import bevy_terrain::atlas
#import bevy_terrain::debug

fn calculate_position(vertex_index: u32, patch: Patch) -> vec2<f32> {
    // use first and last index twice, to form degenerate triangles
    // Todo: documentation
    let row_index = clamp(vertex_index % view_config.vertices_per_row, 1u, view_config.vertices_per_row - 2u) - 1u;
    var vertex_position = vec2<u32>((row_index & 1u) + vertex_index / view_config.vertices_per_row, row_index >> 1u);

#ifndef MESH_MORPH
    // stitch the edges of the patches together
    if (vertex_position.x == 0u && (patch.stitch & 1u) != 0u) {
        vertex_position.y = vertex_position.y & 0xFFFEu; // mod 2
    }
    if (vertex_position.y == 0u && (patch.stitch & 2u) != 0u) {
        vertex_position.x = vertex_position.x & 0xFFFEu; // mod 2
    }
    if (vertex_position.x == view_config.patch_size && (patch.stitch & 4u) != 0u) {
        vertex_position.y = vertex_position.y + 1u & 0xFFFEu; // mod 2
    }
    if (vertex_position.y == view_config.patch_size && (patch.stitch & 8u) != 0u) {
        vertex_position.x = vertex_position.x + 1u & 0xFFFEu; // mod 2
    }
#endif

    var local_position = vec2<f32>((patch.coords * view_config.patch_size + vertex_position)) * f32(patch.size) * view_config.patch_scale;

#ifdef MESH_MORPH
    // Todo: consider finding a way to morph more than one patch size difference
    let world_position = vec3<f32>(local_position.x, view_config.height_under_viewer, local_position.y);
    let viewer_distance = distance(world_position, view.world_position.xyz);
    let morph_distance = f32(patch.size) * view_config.view_distance;
    let morph = clamp(1.0 - (1.0 - viewer_distance / morph_distance) / morph_blend, 0.0, 1.0);

    if (morph > 0.0) {
        let frac_part = ((vec2<f32>(vertex_position) * 0.5) % 1.0) * 2.0;
        local_position = local_position - frac_part * morph * view_config.patch_scale * f32(patch.size);
    }
#endif

    local_position.x = clamp(local_position.x, 0.0, f32(view_config.terrain_size));
    local_position.y = clamp(local_position.y, 0.0, f32(view_config.terrain_size));

    return local_position;
}

fn color_fragment(
    in: FragmentInput,
    lod: u32,
    atlas_index: i32,
    atlas_coords: vec2<f32>
) -> vec4<f32> {
    var color = vec4<f32>(0.0);

    let height_coords = atlas_coords * height_scale + height_offset;
    let albedo_coords = atlas_coords * albedo_scale + albedo_offset;

    #ifndef BRIGHT
        color = mix(color, vec4<f32>(1.0), 0.5);
    #endif

    #ifdef SHOW_LOD
        color = mix(color, show_lod(lod, in.world_position.xyz), 0.4);
    #endif

    #ifdef ALBEDO
        color = mix(color, textureSample(albedo_atlas, filter_sampler, albedo_coords, atlas_index), 0.5);
    #endif

    #ifdef SHOW_UV
        color = mix(color, vec4<f32>(atlas_coords.x, atlas_coords.y, 0.0, 1.0), 0.5);
    #endif

    #ifdef LIGHTING
        let world_normal = calculate_normal(height_coords, atlas_index, lod);

        let ambient = 0.1;
        let direction = normalize(vec3<f32>(3.0, 1.0, -2.0));
        let diffuse = max(dot(direction, world_normal), 0.0);
        color = color * (ambient + diffuse);

        // var pbr_input: PbrInput;
        // pbr_input.material.base_color = color;
        // pbr_input.material.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        // pbr_input.material.perceptual_roughness = 0.089;
        // pbr_input.material.metallic = 0.01;
        // pbr_input.material.reflectance = 0.5;
        // pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE;
        // pbr_input.material.alpha_cutoff = 0.5;
        // pbr_input.occlusion = 1.0;
        // pbr_input.frag_coord = in.frag_coord;
        // pbr_input.world_position = in.world_position;
        // pbr_input.world_normal = world_normal;
        // pbr_input.is_orthographic = view.projection[3].w == 1.0;
        // pbr_input.N = world_normal;
        // pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
        // color = pbr(pbr_input);
    #endif

    return color;
}

[[stage(vertex)]]
fn vertex(vertex: VertexInput) -> FragmentInput {
    let patch_index = vertex.index / view_config.vertices_per_patch;
    let vertex_index = vertex.index % view_config.vertices_per_patch;

    let patch = patches.data[patch_index];
    let local_position = calculate_position(vertex_index, patch);

    let world_position = vec3<f32>(local_position.x, config.height / 2.0, local_position.y);
    let viewer_distance = distance(world_position, view.world_position.xyz);
    let log_distance = log2(2.0 * viewer_distance / view_config.view_distance);
    let ratio = (1.0 - log_distance % 1.0) / vertex_blend;

    let lookup = atlas_lookup(log_distance, local_position);
    var height = height_vertex(lookup.atlas_index, lookup.atlas_coords);

    if (ratio < 1.0) {
        let lookup2 = atlas_lookup(log_distance + 1.0, local_position);
        var height2 = height_vertex(lookup2.atlas_index, lookup2.atlas_coords);
        height = mix(height2, height, ratio);
    }

    let world_position = mesh.model * vec4<f32>(local_position.x, height, local_position.y, 1.0);

    var fragment: FragmentInput;
    fragment.frag_coord = view.view_proj * world_position;
    fragment.local_position = vec2<f32>(local_position);
    fragment.world_position = world_position;
    fragment.color = vec4<f32>(0.0);

#ifdef SHOW_PATCHES
    fragment.color = show_patches(patch, local_position);
#endif

    return fragment;
}

[[stage(fragment)]]
fn fragment(fragment: FragmentInput) -> [[location(0)]] vec4<f32> {
    let viewer_distance = distance(fragment.world_position.xyz, view.world_position.xyz);
    let log_distance = log2(2.0 * viewer_distance / view_config.view_distance);
    let ratio = (1.0 - log_distance % 1.0) / fragment_blend;

    let lookup = atlas_lookup(log_distance, fragment.local_position);
    var color = color_fragment(fragment, lookup.lod, lookup.atlas_index, lookup.atlas_coords);

    if (ratio < 1.0) {
        let lookup2 = atlas_lookup(log_distance + 1.0, fragment.local_position);
        let color2 = color_fragment(fragment, lookup2.lod, lookup2.atlas_index, lookup2.atlas_coords);
        color = mix(color2, color, ratio);
    }

    return mix(fragment.color, color, 0.8);
}
