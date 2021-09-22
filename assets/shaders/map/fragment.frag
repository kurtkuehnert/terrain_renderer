#version 450

const int MAX_POINT_LIGHTS = 10;
const int MAX_LAYER_COUNT = 16;
const float AMBIANT = 0.05;

struct PointLight {
    vec4 pos;
    vec4 color; // color * intensity
    vec4 lightParams; // x = 1/range, y = radius
};

layout(location = 0) in vec3 v_WorldPosition;
layout(location = 1) in vec3 v_WorldNormal;
layout(location = 2) in vec2 v_Uv;
layout(location = 4) in float height;

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

layout(set = 3, binding = 1) uniform MapMaterial {
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
    bool above = CameraPos.y > v_WorldPosition.y;

    // above && height > water_height || 
    
    if (!above && height < water_height - 0.2) {
        discard;
    }

    float height = height / map_height; // normalaize height

    // set color in range [0, layer_height[0]]
    o_Target = colors[0];

    // set color in range [layer_height[i], layer_height[i + 1]]
    for (int i = 0; i < layer_count; i++) {
        // float drawStrength = saturate(sign(height - layer_heights[i])); // update color if vertex is above the layer

        // smoothly interpolate between the diffrent layers
        float drawStrength = inverse_lerp(-blend_values[i] / 2, blend_values[i] / 2, height - layer_heights[i]);

        o_Target = mix(o_Target, colors[i + 1], drawStrength);
    }

    vec3 light_color = vec3(0, 0, 0);

    // combine the lighting color for all point lights
    for (int i = 0; i < NumLights.x && i < MAX_POINT_LIGHTS; ++i) {
        PointLight light = PointLights[i];

        vec3 normalized_light_direction = normalize(v_WorldPosition - light.pos.xyz);
        float brightness_diffuse = max(dot(normalized_light_direction, v_WorldNormal), 0);

        light_color += max((brightness_diffuse + AMBIANT) * light.color.rgb / 100 * o_Target.rgb, vec3(0.0, 0.0, 0.0));
    }

    o_Target = vec4(light_color, o_Target.a);
}
