
#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=0) out vec4 f_color;


layout(set = 0, binding = 0, rgba8) readonly uniform image2D t_image;


void main() {
    ivec2 size = imageSize(t_image);
    f_color = imageLoad(t_image,ivec2(v_tex_coords*size));
}