#version 450

#define PI 3.1415926538

layout(location = 0) in vec3 v_ModelPosition;
layout(location = 1) in vec4 v_WorldPosition;
layout(location = 2) in vec3 v_WorldNormal;
layout(location = 3) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform CameraPosition {
    vec4 CameraPos;
};

layout(set = 2, binding = 0) uniform SkyMaterial {
    vec4 sun_direction;
    vec4 sky_color;
    vec4 horizon_color;
    vec4 sun_color;
    vec4 moon_color;
    float star_count;
    float star_sharpness;
    float sun_size;
    float moon_size;
    float moon_phase;
};

vec2 unity_voronoi_noise_randomVector (vec2 UV, float offset)
{
    mat2x2 m = mat2x2(15.27, 47.63, 99.41, 89.98);
    UV = fract(sin(UV * m) * 46839.32);
    return vec2(sin(UV.y*+offset)*0.5+0.5, cos(UV.x*offset)*0.5+0.5);
}

float Unity_Voronoi_float(vec2 UV, float AngleOffset, float CellDensity)
{
    float Out = 0.0;
    vec2 g = floor(UV * CellDensity);
    vec2 f = fract(UV * CellDensity);
    float t = 8.0;
    vec3 res = vec3(8.0, 0.0, 0.0);

    for(int y=-1; y<=1; y++)
    {
        for(int x=-1; x<=1; x++)
        {
            vec2 lattice = vec2(x,y);
            vec2 offset = unity_voronoi_noise_randomVector(lattice + g, AngleOffset);
            float d = distance(lattice + offset, f);
            if(d < res.x)
            {
                res = vec3(d, offset.x, offset.y);
                Out = res.x;
            }
        }
    }

    return Out;
}

float atan2(in float y, in float x)
{
    bool s = (abs(x) > abs(y));
    return mix(PI/2.0 - atan(x,y), atan(y,x), s);
}

void main() {
    float height = clamp(v_WorldNormal.y, 0, 1);

    vec4 sky_gradiant = mix(horizon_color, sky_color, height) * sign(height); 

    vec2 uv = v_Uv;

    o_Target = sky_gradiant;

    float voronoi = Unity_Voronoi_float(v_Uv, 10.0, star_count);


    float stars = clamp(1 - voronoi, 0, 1);

    stars = pow(stars, star_sharpness);

    vec4 stars_color = vec4(vec3(stars), 1);

    o_Target = vec4(vec3(voronoi), 1);
    o_Target = stars_color;

    vec3 sun_direction = vec3(sun_direction.x, sun_direction.y, sun_direction.z);

    float sun_size = sun_size * sun_size;

    vec3 view_direction = normalize(CameraPos.xyz - v_WorldPosition.xyz);

    float sun = dot(sun_direction, view_direction);

    sun = length(sun_direction - v_WorldNormal);

    sun = 1 - sign(sun - sun_size);

    // sun = dot(vec3(sun_direction.x, sun_direction.y, sun_direction.z), view_direction);

    // sun = acos(sun);

    // sun *= sun_size * sun_size;
    // sun = pow(sun, 100);


    // sun = smoothstep(1-0.02, 1 - 0.01, sun);

    // sun = 1- sun;


    float sky_brightness = smoothstep(-0.4, 0.7, dot(sun_direction, vec3(0, 1, 0)));

    o_Target = mix(stars_color, sky_gradiant, sky_brightness);


    // o_Target = vec4(sun, sun, sun, 1);

    // o_Target = vec4(vec3(sky_brightness), 1);

    // o_Target = vec4((v_WorldNormal + vec3(1)) / 2, 1);

    // o_Target = vec4(sun_size);

    // o_Target = vec4(v_Uv, 0, 1);

    // o_Target = (sun_direction + vec4(1)) / 2;

    // o_Target = vec4(0, 0.5, 1, 1);

    // o_Target = vec4(normalize((v_WorldNormal + normalize(vec3(1, 1, 1)))), 1);
    // o_Target = vec4(v_Uv , 0, 1);
}