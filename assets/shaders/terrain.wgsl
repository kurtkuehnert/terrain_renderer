// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct UniformData {
    height: f32;
};

struct Patch {
    position: vec2<u32>;
    size: u32;
    atlas_index: u32;
    lod: u32;
};

struct PatchList {
    data: array<Patch>;
};

// vertex intput
struct Vertex {
    [[builtin(instance_index)]] index: u32;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

// fragment input
struct Fragment {
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(2)]] world_position: vec4<f32>;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> uniform_data: UniformData;
[[group(2), binding(1)]]
var height_texture: texture_2d<u32>;

[[group(3), binding(2)]]
var<storage, read_write> patch_list: PatchList;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let patch = patch_list.data[vertex.index];
    let patch_position = vec4<f32>(f32(patch.position.x), 0.0, f32(patch.position.y), 1.0);
    var local_position = patch_position + vec4<f32>(vertex.position, 0.0) * f32(patch.size);

    let coords = vec2<i32>(local_position.xz);
    let height = f32(textureLoad(height_texture, coords, 0).r) / 65535.0;

    local_position.y = height * uniform_data.height;

    let world_position = mesh.model * local_position;

    var out: Fragment;
    out.frag_coord = view.view_proj * world_position;
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
    var output_color: vec4<f32> = fragment.color;
    output_color = output_color / pow(length(view.world_position.xyz - fragment.world_position.xyz), 1.5) * 20000.0;
    return output_color;
}
