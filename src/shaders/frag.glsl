#version 140

in vec2 vert_tex_coords;

out vec4 colour;

uniform sampler2D tex;
uniform mat4 colour_lut;

void main() {
    float key = texture(tex, vert_tex_coords).x;
    colour = colour_lut[uint(key * 255.0 + 0.5)];
}
