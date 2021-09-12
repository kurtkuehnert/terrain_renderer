#version 450

layout(location = 0) in vec4 v_ClipPosition;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform texture2DArray WaterMaterial_refraction_texture;
layout(set = 2, binding = 1) uniform sampler WaterMaterial_refraction_texture_sampler;
layout(set = 2, binding = 2) uniform texture2DArray WaterMaterial_reflection_texture;
layout(set = 2, binding = 3) uniform sampler WaterMaterial_reflection_texture_sampler;

void main() {
    vec2 ndc = (v_ClipPosition.xy / v_ClipPosition.w + 1) / 2;
    vec3 refraction_uv = vec3(ndc.x, 1 - ndc.y, 0);
    vec3 reflection_uv = vec3(ndc.x, ndc.y, 0);

    vec4 refraction_color = texture(
        sampler2DArray(WaterMaterial_refraction_texture, WaterMaterial_refraction_texture_sampler), refraction_uv);
    vec4 reflection_color = texture(
        sampler2DArray(WaterMaterial_reflection_texture, WaterMaterial_reflection_texture_sampler), reflection_uv);

    o_Target = mix(refraction_color, reflection_color, 0.4);
}