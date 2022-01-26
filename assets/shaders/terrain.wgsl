// imports the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct Material {
    height: f32;
};







[[group(1), binding(0)]]
var<uniform> material: Material;
[[group(1), binding(1)]]
var height_texture: texture_2d<u32>;
[[group(1), binding(2)]]
var height_sampler: sampler;

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

// vertex intput
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;

    [[location(3)]] tile_position: vec2<u32>;
    [[location(4)]] tile_size: u32;
    [[location(5)]] tile_range: f32;
    [[location(6)]] tile_color: vec4<f32>;

};

// fragment input
struct Fragment {
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] tile_color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] world_position: vec4<f32>;

};



// the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let tile_position = vec4<f32>(f32(vertex.tile_position.x), 0.0, f32(vertex.tile_position.y), 1.0);
    var local_position = tile_position + vec4<f32>(vertex.position, 0.0) * f32(vertex.tile_size);

    let uv: vec2<f32> = (local_position.xz + 0.5) / 1024.0;
    let coords = vec2<i32>(local_position.xz);

    // let height = textureSample(height_texture, height_sampler, vec2<f32>(0.0, 0.0)).r;
    let height = f32(textureLoad(height_texture, coords, 0).r) / 65535.0;

    local_position.y = height * material.height;

    let world_position = mesh.model * local_position;

    var out: Fragment;
    out.frag_coord = view.view_proj * world_position;
    out.tile_color = vertex.tile_color;
    // out.tile_color = vec4<f32>(uv, 0.0, 1.0);
    out.uv = uv;
    out.world_position = world_position;
    return out;
}


// the fragment shader
[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    var output_color: vec4<f32> = fragment.tile_color;

    // output_color = output_color / pow(length(view.world_position.xyz - fragment.world_position.xyz), 1.5) * 10000.0;
    // output_color = vec4<f32>(calculate_normal(fragment.uv), 1.0);

    return output_color;
}
