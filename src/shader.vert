#version 450

layout(location=0) in vec3 a_position;

layout(location=0) out vec2 v_tex_coords;

void main() {
    v_tex_coords = (a_position.xy+1.0)/2.0;
    v_tex_coords.y = 1 - v_tex_coords.y;
    gl_Position = vec4(a_position, 1.0);
}