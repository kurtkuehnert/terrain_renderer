#version 450

const int MAX_POINT_LIGHTS = 10;
const int MAX_LAYER_COUNT = 16;
const float AMBIANT = 0.05;

struct PointLight {
    vec4 pos;
    vec4 color; // color * intensity
    vec4 lightParams; // x = 1/range, y = radius
};

layout(location = 0) in vec3 frag_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in float height;
layout(location = 3) in float scale;
layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 0, binding = 1) uniform CameraPosition {
    vec4 CameraPos;
};

layout(set = 2, binding = 0) uniform Lights {
    vec4 AmbientColor;
    uvec4 NumLights; // x = point lights
    PointLight PointLights[MAX_POINT_LIGHTS];
};

layout(set = 3, binding = 2) uniform MapMaterial_appearance {
    vec4[MAX_LAYER_COUNT] colors;
// uses array of float but has an alignement of vec4
    float[MAX_LAYER_COUNT] layer_heights;
    float[MAX_LAYER_COUNT] blend_values;
    float map_height;
    float water_height;
    int layer_count;
};

float saturate(float value) {
    return clamp(value, 0, 1);
}

float inverse_lerp(float a, float b, float value) {
    return saturate((value - a) / (b - a));
}

void main() {
    // o_Target = vec4(scale, 0, height, 1);

    bool above = CameraPos.y > frag_pos.y;

    // above && height > water_height ||

    if (!above && height * map_height < water_height) {
        discard;
    }

    float height = height; // normalaize height

    // set color in range [0, layer_height[0]]
    o_Target = colors[0];

    // set color in range [layer_height[i], layer_height[i + 1]]
    for (int i = 0; i < layer_count; i++) {
        // float drawStrength = saturate(sign(height - layer_heights[i])); // update color if vertex is above the layer

        // smoothly interpolate between the diffrent layers
        float drawStrength = inverse_lerp(-blend_values[i] / 2, blend_values[i] / 2, height - layer_heights[i]);

        o_Target = mix(o_Target, colors[i + 1], drawStrength);
    }

    // o_Target = vec4(scale, 0, height, 1);





    vec3 light_color = vec3(0, 0, 0);

    // combine the lighting color for all point lights
    for (int i = 0; i < NumLights.x && i < MAX_POINT_LIGHTS; ++i) {
        PointLight light = PointLights[i];

        vec3 normalized_light_direction = normalize(frag_pos - light.pos.xyz);
        float brightness_diffuse = max(dot(normalized_light_direction, frag_normal.xyz), 0);
        // o_Target = vec4(0,0,brightness_diffuse, 1);
        // o_Target = vec4(normalized_light_direction, 1);

        // o_Target = vec4(frag_normal.xyz, o_Target.a);


        light_color += max((brightness_diffuse + AMBIANT) * light.color.rgb / 100 * o_Target.rgb, vec3(0.0, 0.0, 0.0));
    }

    o_Target = vec4(light_color, o_Target.a);

    //

}
