#version 450

const int MAX_LAYER_COUNT = 5;

layout(location = 0) in vec3 Vertex_Position;
layout(location = 0) out float height;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    height = Vertex_Position.y;
}