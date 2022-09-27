#import bevy_terrain::types

struct TerrainConfig {
    lod_count: u32,
    height: f32,
    chunk_size: u32,
    terrain_size: u32,

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
var terrain_sampler: sampler;
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

#import bevy_terrain::atlas
#import bevy_terrain::functions
#import bevy_terrain::debug

struct FragmentData {
    world_normal: vec3<f32>,
    color: vec4<f32>,
    dis: bool
}

fn lookup_fragment_data(in: FragmentInput, lookup: AtlasLookup) -> FragmentData {
    let lod = lookup.lod;
    let atlas_index = lookup.atlas_index;
    let atlas_coords = lookup.atlas_coords;

    let height_coords = atlas_coords * config.height_scale + config.height_offset;
    let albedo_coords = atlas_coords * config.albedo_scale + config.albedo_offset;

#ifdef VERTEX_NORMAL
    let world_normal = in.world_normal;
#else
    let world_normal = calculate_normal(height_coords, atlas_index, lod);
#endif

    var color = vec4<f32>(0.0);
    var dis = false;

#ifdef ALBEDO
    color = textureSample(albedo_atlas, terrain_sampler, albedo_coords, atlas_index);

   #ifdef TEST3
       dis = dot(color, vec4<f32>(1.0)) == 4.0 || dot(color.xyz, vec3<f32>(1.0)) == 0.0;
   #endif
#endif

#ifndef BRIGHT
    color = mix(color, vec4<f32>(1.0), 0.5);
#endif

#ifdef SHOW_LOD
    color = mix(color, show_lod(lod, in.world_position.xyz), 0.4);
#endif

#ifdef SHOW_UV
    color = mix(color, vec4<f32>(atlas_coords.x, atlas_coords.y, 0.0, 1.0), 0.5);
#endif

    return FragmentData(world_normal, color, dis);
}

fn blend_fragment_data(data1: FragmentData, data2: FragmentData, blend_ratio: f32) -> FragmentData {
    let world_normal = mix(data2.world_normal, data1.world_normal, blend_ratio);
    let color = mix(data2.color, data1.color, blend_ratio);
    let dis = data2.dis || data1.dis;

    return FragmentData(world_normal, color, dis);
}

fn fragment_color(in: FragmentInput, data: FragmentData) -> vec4<f32> {
    let world_normal = data.world_normal;
    var color = data.color;

    #ifdef LIGHTING
        var pbr_input: PbrInput = pbr_input_new();
        pbr_input.material.base_color = color;
        pbr_input.material.perceptual_roughness = 1.0;
        pbr_input.material.reflectance = 0.0;
        pbr_input.frag_coord = in.frag_coord;
        pbr_input.world_position = in.world_position;
        pbr_input.world_normal = world_normal;
        pbr_input.is_orthographic = view.projection[3].w == 1.0;
        pbr_input.N = world_normal;
        pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

        color = tone_mapping(pbr(pbr_input));
    #endif

    return color;
}

@fragment
fn fragment(fragment: FragmentInput) -> FragmentOutput {
    if (fragment.local_position.x < 2.0 || fragment.local_position.x > f32(config.terrain_size) - 2.0 ||
        fragment.local_position.y < 2.0 || fragment.local_position.y > f32(config.terrain_size) - 2.0) {
        discard;
    }

    let blend = calculate_blend(fragment.world_position.xyz, view_config.fragment_blend);

    let lookup = atlas_lookup(blend.lod, fragment.local_position);
    var fragment_data = lookup_fragment_data(fragment, lookup);

    if (blend.ratio < 1.0) {
        let lookup2 = atlas_lookup(blend.lod + 1u, fragment.local_position);
        let fragment_data2 = lookup_fragment_data(fragment, lookup2);

        fragment_data = blend_fragment_data(fragment_data, fragment_data2, blend.ratio);
    }

    if (fragment_data.dis) {
        discard;
    }

    var color = fragment_color(fragment, fragment_data);

    color = mix(color, vec4<f32>(fragment.color.xyz, 1.0), fragment.color.w);

    return FragmentOutput(color);
}

#ifndef MINMAX
#import bevy_terrain::vertex
#else
#import bevy_terrain::minmax
#endif
