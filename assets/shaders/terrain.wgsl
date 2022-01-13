// imporst the View struct and the view binding, aswell as the lighting structs and bindings
#import bevy_pbr::mesh_view_bind_group
// imports the Mesh struct, without the binding
#import bevy_pbr::mesh_struct


struct TerrainMaterial {
    color: vec4<f32>;
};

// the bindings for the material and the mesh
[[group(1), binding(0)]]
var<uniform> material: TerrainMaterial;
[[group(2), binding(0)]]
var<uniform> mesh: Mesh;


// vertex intput
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

// fragment input
struct Fragment {
    [[builtin(position)]] clip_position: vec4<f32>;
};


// the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> Fragment {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);

    var out: Fragment;
    out.clip_position = view.view_proj * world_position;
    return out;
}


// the fragment shader
[[stage(fragment)]]
fn fragment(fragment: Fragment) -> [[location(0)]] vec4<f32> {
    return material.color;
}
