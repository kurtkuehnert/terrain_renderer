// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct UniformData {
    height: f32;
};

struct PatchData {
    position: vec2<u32>;
    size: u32;
    range: f32;
    color: vec4<f32>;
};

struct PatchBuffer {
    data: array<PatchData>;
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
    [[location(0)]] patch_color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] world_position: vec4<f32>;
};

// mesh bindings
[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

// terrain data bindings
[[group(2), binding(0)]]
var<uniform> uniform_data: UniformData;
[[group(2), binding(1)]]
var<storage> patch_buffer: PatchBuffer;
[[group(2), binding(2)]]
var height_texture: texture_2d<u32>;
[[group(2), binding(3)]]
var height_sampler: sampler;

// the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let patch_data = patch_buffer.data[vertex.index];

    let patch_position = vec4<f32>(f32(patch_data.position.x), 0.0, f32(patch_data.position.y), 1.0);
    var local_position = patch_position + vec4<f32>(vertex.position, 0.0) * f32(patch_data.size);

    let uv: vec2<f32> = (local_position.xz + 0.5) / 1024.0;
    let coords = vec2<i32>(local_position.xz);

    let height = f32(textureLoad(height_texture, coords, 0).r) / 65535.0;

    local_position.y = height * uniform_data.height;

    let world_position = mesh.model * local_position;

    var out: Fragment;
    out.frag_coord = view.view_proj * world_position;
    out.patch_color = patch_data.color;
    out.uv = uv;
    out.world_position = world_position;
    return out;
}


// the fragment shader
[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    var output_color: vec4<f32> = fragment.patch_color;

    // output_color = output_color / pow(length(view.world_position.xyz - fragment.world_position.xyz), 1.5) * 10000.0;
    // output_color = vec4<f32>(calculate_normal(fragment.uv), 1.0);

    return output_color;
}
