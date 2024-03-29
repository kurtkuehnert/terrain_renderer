#import bevy_terrain::types

struct TerrainConfig {
    lod_count: u32,
    height: f32,
    leaf_node_size: u32,
    terrain_size: u32,

    height_size: f32,
    minmax_size: f32,
    albedo_size: f32,
    _empty: f32,
    height_scale: f32,
    minmax_scale: f32,
    albedo_scale: f32,
    _empty: u32,
    height_offset: f32,
    minmax_offset: f32,
    albedo_offset: f32,
    _empty: u32,
}

// view bindings
#import bevy_pbr::mesh_view_bindings

// terrain view bindings
@group(1) @binding(0)
var<uniform> view_config: TerrainViewConfig;
@group(1) @binding(1)
var quadtree: texture_2d_array<u32>;
@group(1) @binding(2)
var<storage> tiles: TileList;

// terrain bindings
@group(2) @binding(0)
var<uniform> config: TerrainConfig;
@group(2) @binding(1)
var atlas_sampler: sampler;
#ifndef TEST1
@group(2) @binding(2)
var height_atlas: texture_2d_array<f32>;
@group(2) @binding(3)
var minmax_atlas: texture_2d_array<f32>;
#else
@group(2) @binding(3)
var height_atlas: texture_2d_array<f32>;
@group(2) @binding(2)
var minmax_atlas: texture_2d_array<f32>;
#endif
#ifdef ALBEDO
@group(2) @binding(4)
var albedo_atlas: texture_2d_array<f32>;
#endif

#import bevy_pbr::mesh_types
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

#import bevy_terrain::node
#import bevy_terrain::functions
#import bevy_terrain::debug

struct FragmentData {
    world_normal: vec3<f32>,
    color: vec4<f32>,
}

fn vertex_height(lookup: NodeLookup) -> f32 {
    let height_coords = lookup.atlas_coords * config.height_scale + config.height_offset;
    var height = textureSampleLevel(height_atlas, atlas_sampler, height_coords, lookup.atlas_index, 0.0).x;

#ifdef TEST3
    // still produces bugs for nodes of mip 1 and above (preprocessing)
    let gather = textureGather(0, height_atlas, atlas_sampler, height_coords, lookup.atlas_index);

    if (gather.x == 0.0 || gather.y == 0.0 || gather.z == 0.0 || gather.w == 0.0) {
        height = height / 0.0;
    }
#endif

    return height * config.height;
}

fn lookup_fragment_data(input: FragmentInput, lookup: NodeLookup, ddx: vec2<f32>, ddy: vec2<f32>) -> FragmentData {
    let atlas_lod = lookup.atlas_lod;
    let atlas_index = lookup.atlas_index;
    let atlas_coords = lookup.atlas_coords;
    let ddx = ddx / f32(1u << atlas_lod);
    let ddy = ddy / f32(1u << atlas_lod);

    let height_coords = atlas_coords * config.height_scale + config.height_offset;
    let height_ddx = ddx / config.height_size;
    let height_ddy = ddy / config.height_size;
    let albedo_coords = atlas_coords * config.albedo_scale + config.albedo_offset;
    let albedo_ddx = ddx / config.albedo_size;
    let albedo_ddy = ddy / config.albedo_size;

    let world_normal = calculate_normal(height_coords, atlas_index, atlas_lod, height_ddx, height_ddy);

    var color = vec4<f32>(0.0);

#ifdef ALBEDO
#ifdef SAMPLE_GRAD
    color = textureSampleGrad(albedo_atlas, atlas_sampler, albedo_coords, atlas_index, albedo_ddx, albedo_ddy);
#else
    // var color = textureSampleBias(albedo_atlas, atlas_sampler, albedo_coords, atlas_index, 3.0);
    // var color = textureSample(albedo_atlas, atlas_sampler, albedo_coords, atlas_index);
    color = textureSampleLevel(albedo_atlas, atlas_sampler, albedo_coords, atlas_index, 0.0);
#endif
#endif

#ifdef BRIGHT
    color = mix(color, vec4<f32>(1.0), 0.5);
#endif

#ifdef SHOW_LOD
    color = mix(color, show_lod(atlas_lod, input.world_position.xyz), 0.4);
#endif

#ifdef SHOW_UV
    color = mix(color, vec4<f32>(atlas_coords.x, atlas_coords.y, 0.0, 1.0), 0.5);
#endif

    return FragmentData(world_normal, color);
}

fn blend_fragment_data(data1: FragmentData, data2: FragmentData, blend_ratio: f32) -> FragmentData {
    let world_normal = mix(data2.world_normal, data1.world_normal, blend_ratio);
    let color = mix(data2.color, data1.color, blend_ratio);

    return FragmentData(world_normal, color);
}

fn process_fragment(input: FragmentInput, data: FragmentData) -> Fragment {
    var color = mix(data.color, vec4<f32>(input.debug_color.xyz, 1.0), input.debug_color.w);

#ifdef LIGHTING
    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.base_color = color;
    pbr_input.material.perceptual_roughness = 1.0;
    pbr_input.material.reflectance = 0.0;
    pbr_input.frag_coord = input.frag_coord;
    pbr_input.world_position = input.world_position;
    pbr_input.world_normal = data.world_normal;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.N = data.world_normal;
    pbr_input.V = calculate_view(input.world_position, pbr_input.is_orthographic);

    color = tone_mapping(pbr(pbr_input));
#endif

    return Fragment(color, false);
}

#ifndef MINMAX
#import bevy_terrain::vertex
#else
#import bevy_terrain::minmax
#endif

#import bevy_terrain::fragment