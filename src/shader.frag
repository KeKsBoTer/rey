
#version 450

layout(location = 0) in vec2 v_tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0, rgba16f) readonly uniform image2D t_image;

vec3 lessThan(vec3 f, float value) {
    return vec3((f.x < value) ? 1.0f : 0.0f, (f.y < value) ? 1.0f : 0.0f,
                (f.z < value) ? 1.0f : 0.0f);
}

// source https://www.shadertoy.com/view/tdXBW8
vec3 linearToSRGB(vec3 rgb) {
    rgb = clamp(rgb, 0.0f, 1.0f);

    return mix(pow(rgb * 1.055f, vec3(1.f / 2.4f)) - 0.055f, rgb * 12.92f,
               lessThan(rgb, 0.0031308f));
}

void main() {
    ivec2 size = imageSize(t_image);
    f_color = imageLoad(t_image, ivec2(v_tex_coords * size));
    f_color.rgb = linearToSRGB(f_color.rgb);
}