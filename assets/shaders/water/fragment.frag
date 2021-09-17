#version 450

const int MAX_POINT_LIGHTS = 10;

const float SHINE_DAMPER = 25; // determins the shinieness of the water
const float REFLECTIVITY = 0.005; // reflectivity of the water surface

const float DISTORTION_OFFSET = 0.12; // half of max surface distortion
const float NORMAL_OFFSET = 0.33; // half of max surface normal (x, z)
const float REFLECTIVENESS = 0.6; // balances the 
const float DISTORTION_MULTIPLIER = 0.1; // scales down the effect of the distortion
const float DISTORTION_THRESHOLD = 80; // the depth threshold until which the distortion is scaled down

struct PointLight {
    vec4 pos;
    vec4 color; // color * intensity
    vec4 lightParams; // x = 1/range, y = radius
};

layout(location = 0) in vec4 v_ClipPosition;
layout(location = 1) in vec2 v_TexturePosition;
layout(location = 2) in vec4 v_WorldPosition;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform CameraPosition {
    vec4 CameraPos;
};

layout(set = 2, binding = 0) uniform Lights {
    vec4 AmbientColor;
    uvec4 NumLights; // x = point lights
    PointLight PointLights[MAX_POINT_LIGHTS];
};

layout(set = 3, binding = 0) uniform texture2DArray WaterMaterial_refraction_texture;
layout(set = 3, binding = 1) uniform sampler WaterMaterial_refraction_texture_sampler;
layout(set = 3, binding = 2) uniform texture2DArray WaterMaterial_reflection_texture;
layout(set = 3, binding = 3) uniform sampler WaterMaterial_reflection_texture_sampler;
layout(set = 3, binding = 4) uniform texture2DArray WaterMaterial_depth_texture;
layout(set = 3, binding = 5) uniform sampler WaterMaterial_depth_texture_sampler;
layout(set = 3, binding = 6) uniform texture2DArray WaterMaterial_dudv_texture;
layout(set = 3, binding = 7) uniform sampler WaterMaterial_dudv_texture_sampler;
layout(set = 3, binding = 8) uniform texture2DArray WaterMaterial_normal_texture;
layout(set = 3, binding = 9) uniform sampler WaterMaterial_normal_texture_sampler;
layout(set = 3, binding = 10) uniform WaterMaterial_wave_uniform {
    float wave_sparsity;
    float wave_strength;
    float wave_cycle;
};

float calculate_distance(float x, float near, float far) {
    return 2.0 * near * far / (far + near - (2.0 * x - 1.0) * (far - near));
}

void main() {
    vec2 ndc = (v_ClipPosition.xy / v_ClipPosition.w + 1) / 2;

    // Todo: load as uniform
    float near = 1;
    float far = 4000;

    // calculate the depth (distance between surface and floor in view direction)
    float surface_distance = calculate_distance(gl_FragCoord.z, near, far);
    float floor_distance = texture(sampler2DArray(WaterMaterial_depth_texture, WaterMaterial_depth_texture_sampler), vec3(ndc.x, 1 - ndc.y, 0)).x;
    floor_distance = calculate_distance(floor_distance , near, far);
    float depth = (floor_distance - surface_distance);

    // dampen the distortion near the edges to reduce the problem of glitchy edges
    float edge_damper = clamp(depth / DISTORTION_THRESHOLD, 0, 1) * wave_strength;

    // calculate the texture uv within the current water tile (tile size = wave sparsity)
    vec2 texture_uv = mod(v_TexturePosition, wave_sparsity) / wave_sparsity;

    // apply the first offset to the texture uv, and sample the intermediate distortion form it
    vec2 intermediate_distortion_uv = texture_uv + vec2(wave_cycle, -wave_cycle);
    vec2 intermediate_distortion = texture(sampler2DArray(WaterMaterial_dudv_texture,WaterMaterial_dudv_texture_sampler), vec3(intermediate_distortion_uv, 0)).xy;
    // apply the second offset (orthagonally to the first one) to the texture uv 
    // and add the intermediate distortion (scaled down)
    vec2 sample_distortion_uv = texture_uv + vec2(-wave_cycle, wave_cycle) + intermediate_distortion * DISTORTION_MULTIPLIER;

    // compute the distortion on the water surface
    vec2 surface_distortion = (texture(sampler2DArray(WaterMaterial_dudv_texture, WaterMaterial_dudv_texture_sampler), vec3(sample_distortion_uv, 0)).xy);
    surface_distortion -= vec2(DISTORTION_OFFSET);
    surface_distortion *= wave_strength * DISTORTION_MULTIPLIER * edge_damper;

    // compute the normals on the water surface
    vec3 surface_normal = texture(sampler2DArray(WaterMaterial_normal_texture, WaterMaterial_normal_texture_sampler), vec3(sample_distortion_uv, 0)).xzy;
    surface_normal -= vec3(NORMAL_OFFSET, 0, NORMAL_OFFSET);
    surface_normal = normalize(mix(vec3(0, 1, 0), surface_normal, wave_strength));

    // sample the refraction and reflection texture at their screen uvs offset by the surface distortion
    vec2 refraction_uv = vec2(ndc.x, 1 - ndc.y) + surface_distortion;
    vec2 reflection_uv = vec2(ndc.x, ndc.y) + surface_distortion;
    vec4 refraction_color = texture(sampler2DArray(WaterMaterial_refraction_texture, WaterMaterial_refraction_texture_sampler), vec3(refraction_uv, 0));
    vec4 reflection_color = texture(sampler2DArray(WaterMaterial_reflection_texture, WaterMaterial_reflection_texture_sampler), vec3(reflection_uv, 0));

    vec3 surface_pos = v_WorldPosition.xyz;
    vec3 camera_pos = CameraPos.xyz;
    vec3 light_pos = PointLights[0].pos.xyz;
    vec3 light_color = PointLights[0].color.xyz;

    vec3 light_direction = normalize(surface_pos - light_pos);
    vec3 view_direction = normalize(camera_pos - surface_pos);

    // compute specular highlights
    vec3 reflected_light = reflect(light_direction, surface_normal);
	float specular = max(dot(reflected_light, view_direction), 0.0);
	specular = pow(specular, SHINE_DAMPER);
	vec3 specular_highlights = light_color * specular * REFLECTIVITY;

    // compute fresnel effect
    float refractive_factor = clamp(dot(view_direction, surface_normal), 0, 1);
    refractive_factor = pow(refractive_factor, REFLECTIVENESS);

    o_Target = mix(reflection_color, refraction_color, refractive_factor) + vec4(specular_highlights, 0);

    // o_Target = reflection_color;
    // o_Target = refraction_color;
    // o_Target = vec4(surface_normal, 1);
    // o_Target = vec4(surface_distortion * 10, 0, 1);
    // o_Target = vec4(sample_distortion_uv, 0, 1);
    // o_Target = vec4(intermediate_distortion, 0, 1);
    // o_Target = vec4(intermediate_distortion_uv, 0, 1);
    // o_Target = vec4(texture_uv, 0, 1);

    // o_Target = vec4(refractive_factor, 0, 0, 1);
    // o_Target = vec4(surface_normal, 1);
    // o_Target = vec4(depth / 100, 0, 0, 1);
    // o_Target = vec4(edge_damper, 0, 0, 1);

    // vec4 ref = texture(sampler2DArray(WaterMaterial_reflection_texture, WaterMaterial_reflection_texture_sampler), vec3(ndc.x, ndc.y, 0));
    // o_Target = ref;
}