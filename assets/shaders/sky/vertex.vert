#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec3 v_ModelPosition;
layout(location = 1) out vec4 v_WorldPosition;
layout(location = 2) out vec3 v_WorldNormal;
layout(location = 3) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    v_ModelPosition = Vertex_Position;
    v_WorldPosition = Model * vec4(Vertex_Position, 1);
    v_WorldNormal = mat3(Model) * Vertex_Normal;
    v_Uv = Vertex_Uv;
    gl_Position = ViewProj * v_WorldPosition;
}