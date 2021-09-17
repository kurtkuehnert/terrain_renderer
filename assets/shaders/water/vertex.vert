#version 450

layout(location = 0) in vec3 Vertex_Position;

layout(location = 0) out vec4 v_ClipPosition;
layout(location = 1) out vec2 v_TexturePosition;
layout(location = 2) out vec4 v_WorldPosition;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_TexturePosition = Vertex_Position.xz;
    v_WorldPosition = Model * vec4(Vertex_Position, 1.0);
    v_ClipPosition = ViewProj * v_WorldPosition;
    gl_Position = v_ClipPosition;
}