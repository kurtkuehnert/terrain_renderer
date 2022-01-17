// imporst the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

[[group(1), binding(0)]]
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
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tile_color: vec4<f32>;

};


// the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let tile_position = vec4<f32>(f32(vertex.tile_position.x), 0.0, f32(vertex.tile_position.y), 1.0);
    let local_position = tile_position + vec4<f32>(vertex.position, 0.0) * f32(vertex.tile_size);

    let world_position = mesh.model * local_position;

    var out: Fragment;
    out.clip_position = view.view_proj * world_position;
    out.tile_color = vertex.tile_color;
    return out;
}


// the fragment shader
[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    return fragment.tile_color;
}
