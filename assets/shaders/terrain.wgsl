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
let fragment_blend: f32 = 0.8;

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

#import bevy_terrain::utils
#import bevy_terrain::debug

fn height_vertex(atlas_index: i32, atlas_coords: vec2<f32>) -> f32 {
    let height_coords = atlas_coords * height_scale + height_offset;
    return config.height * textureSampleLevel(height_atlas, filter_sampler, height_coords, atlas_index, 0.0).x;
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

        let ambient = 0.3;
        let direction = normalize(vec3<f32>(3.0, 1.0, -2.0));
        let diffuse = max(dot(direction, world_normal), 0.0);
        color = color * (ambient + diffuse);

        // var pbr_input: PbrInput = pbr_input_new();
        // pbr_input.material.base_color = color;
        // pbr_input.frag_coord = in.frag_coord;
        // pbr_input.world_position = in.world_position;
        // pbr_input.world_normal = world_normal;
        // pbr_input.is_orthographic = view.projection[3].w == 1.0;
        // pbr_input.N = world_normal;
        // pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
        // color = vec4<f32>(pbr_input.V, 1.0);
        // color = pbr(pbr_input);
    #endif

    return color;
}

fn calculate_position(vertex_index: u32, patch: Patch, vertices_per_row: u32, patch_lod: u32, patch_size: u32) -> vec2<f32> {
    // use first and last index twice, to form degenerate triangles
    // Todo: documentation
    let row_index = clamp(vertex_index % vertices_per_row, 1u, vertices_per_row - 2u) - 1u;
    var vertex_position = vec2<u32>((row_index & 1u) + vertex_index / vertices_per_row, row_index >> 1u);

    var lod_diff = patch.lod_diff;

    // // stitch the edges of the patches together
    // if (vertex_position.x == 0u) {
    //     let offset = 1u << ((patch.stitch >>  0u) & 0x000Fu);
    //     // if (lod_diff != 0u) {
    //     //     vertex_position.y = ((vertex_position.y + (offset >> 1u) - 1u) / offset) * offset;
    //     // }
    //     vertex_position.y = (vertex_position.y / offset) * offset;
    //     lod_diff = (patch.morph >>  0u) & 0x000Fu;
    // }
    // if (vertex_position.y == 0u) {
    //     let offset = 1u << ((patch.stitch >>  8u) & 0x000Fu);
    //     // if (lod_diff != 0u) {
    //     //     vertex_position.x = ((vertex_position.x + (offset >> 1u) - 1u) / offset) * offset;
    //     // }
    //     vertex_position.x = (vertex_position.x / offset) * offset;
    //     lod_diff = (patch.morph >>  8u) & 0x000Fu;
    // }
    // if (vertex_position.x == patch_size) {
    //     let offset = 1u << ((patch.stitch >> 16u) & 0x000Fu);
    //     // vertex_position.y = ((vertex_position.y + (offset >> 1u)) / offset) * offset;
    //     vertex_position.y = (vertex_position.y / offset) * offset;
    //     lod_diff = (patch.morph >> 16u) & 0x000Fu;
    // }
    // if (vertex_position.y == patch_size) {
    //     let offset = 1u << ((patch.stitch >> 24u) & 0x000Fu);
    //     // vertex_position.x = ((vertex_position.x + (offset >> 1u)) / offset) * offset;
    //     vertex_position.x = (vertex_position.x / offset) * offset;
    //     lod_diff = (patch.morph >> 24u) & 0x000Fu;
    // }

    var local_position = (vec2<f32>(patch.coords) + vec2<f32>(vertex_position) / f32(patch_size)) * f32(patch.size) * view_config.patch_scale;

#ifdef MESH_MORPH
    let morph = calculate_morph(local_position, patch);
    let frac_part = vec2<f32>(vertex_position % vec2<u32>(1u << lod_diff)) / f32(patch_size);
    local_position = local_position - frac_part * morph * f32(patch.size) * view_config.patch_scale;
#endif

    local_position.x = clamp(local_position.x, 0.0, f32(view_config.terrain_size));
    local_position.y = clamp(local_position.y, 0.0, f32(view_config.terrain_size));

    return local_position;
}

[[stage(vertex)]]
fn vertex(vertex: VertexInput) -> VertexOutput {
    var patch_lod = 0u;
    for (; patch_lod < 4u; patch_lod = patch_lod + 1u) {
        if (vertex.index < patches.counts[patch_lod].y) {
            break;
        }
    }

    let patch_size = 2u << patch_lod;
    let vertices_per_row = (patch_size + 2u) << 1u;
    let vertices_per_patch = vertices_per_row * patch_size;

    let patch_index  = (vertex.index - patches.counts[patch_lod].x) / vertices_per_patch + patch_lod * 100000u;
    let vertex_index = (vertex.index - patches.counts[patch_lod].x) % vertices_per_patch;

    // let patch_index = vertex.index / view_config.vertices_per_patch;
    // let vertex_index = vertex.index % view_config.vertices_per_patch;

    let patch = patches.data[patch_index];
    let local_position = calculate_position(vertex_index, patch, vertices_per_row, patch_lod, patch_size);

    let world_position = vec3<f32>(local_position.x, view_config.height_under_viewer, local_position.y);
    let blend = calculate_blend(world_position, vertex_blend);

    let lookup = atlas_lookup(blend.log_distance, local_position);
    var height = height_vertex(lookup.atlas_index, lookup.atlas_coords);

    if (blend.ratio < 1.0) {
        let lookup2 = atlas_lookup(blend.log_distance + 1.0, local_position);
        var height2 = height_vertex(lookup2.atlas_index, lookup2.atlas_coords);
        height = mix(height2, height, blend.ratio);
    }

    var output = vertex_output(local_position, height);

#ifdef SHOW_PATCHES
    output.color = show_patches(patch, local_position, patch_lod);
#endif

    return output;
}

[[stage(fragment)]]
fn fragment(fragment: FragmentInput) -> [[location(0)]] vec4<f32> {
    let blend = calculate_blend(fragment.world_position.xyz, fragment_blend);

    let lookup = atlas_lookup(blend.log_distance, fragment.local_position);
    var color = color_fragment(fragment, lookup.lod, lookup.atlas_index, lookup.atlas_coords);

    if (blend.ratio < 1.0) {
        let lookup2 = atlas_lookup(blend.log_distance + 1.0, fragment.local_position);
        let color2 = color_fragment(fragment, lookup2.lod, lookup2.atlas_index, lookup2.atlas_coords);
        color = mix(color2, color, blend.ratio);
    }

    return mix(fragment.color, color, 0.8);
}
