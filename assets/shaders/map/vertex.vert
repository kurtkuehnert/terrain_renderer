#version 450

const int MAX_LAYER_COUNT = 16;
const vec2 SIZE = vec2(2,0); // twice the grid spacing
const ivec3 OFFSET = ivec3(-1,0,1);

layout(location = 0) in vec3 Vertex_Position;

layout(location = 0) out vec3 frag_pos;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out float height;
layout(location = 3) out float scale;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 0, binding = 1) uniform CameraPosition {
    vec4 CameraPos;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};


layout(set = 3, binding = 0) uniform texture2D MapMaterial_height_map;
layout(set = 3, binding = 1) uniform sampler MapMaterial_height_map_sampler;
layout(set = 3, binding = 2) uniform MapMaterial_appearance {
    vec4[MAX_LAYER_COUNT] colors;
// uses array of float but has an alignement of vec4
    float[MAX_LAYER_COUNT] layer_heights;
    float[MAX_LAYER_COUNT] blend_values;
    float map_height;
    float water_height;
    int layer_count;
};

float get_height(vec4 sampled_height) {
    return (sampled_height.x * 256 + sampled_height.y) / 256.0 * map_height;
}

vec4 calculate_bump(vec2 sample_pos) {
    //     10
    //     |
    // 01--11--21
    //     |
    //     12
    float s11 = get_height(texture(sampler2D(MapMaterial_height_map, MapMaterial_height_map_sampler), sample_pos));
    float s01 = get_height(textureOffset(sampler2D(MapMaterial_height_map, MapMaterial_height_map_sampler), sample_pos, OFFSET.xy));
    float s21 = get_height(textureOffset(sampler2D(MapMaterial_height_map, MapMaterial_height_map_sampler), sample_pos, OFFSET.zy));
    float s10 = get_height(textureOffset(sampler2D(MapMaterial_height_map, MapMaterial_height_map_sampler), sample_pos, OFFSET.yx));
    float s12 = get_height(textureOffset(sampler2D(MapMaterial_height_map, MapMaterial_height_map_sampler), sample_pos, OFFSET.yz));

    vec3 normal = vec3(s21 - s01, -2, s12 - s10) / 2;

    vec4 bump = vec4(normal, s11);

    return bump;
}

void main() {
    vec4 world_position = Model * vec4(Vertex_Position, 1.0);

    scale = world_position.y;
    // scale = 2;
    scale = 16; // 2 * level

    vec2 snapped_pos = round(CameraPos.xz / scale) * scale;

    world_position.xz += snapped_pos;

    vec2 sample_pos = world_position.xz / vec2(512, 512) + vec2(0.5, 0.5);

    vec4 bump = calculate_bump(sample_pos);

    height = bump.w;
    frag_normal = bump.xyz;

    world_position.y = height;

    height /= map_height;

    scale = scale / 160;

    frag_pos = world_position.xyz;

    gl_Position = ViewProj * world_position;
}
