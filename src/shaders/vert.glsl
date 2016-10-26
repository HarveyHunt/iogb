#version 140

in vec2 pos;
in vec2 tex_coords;

out vec2 vert_tex_coords;

uniform mat4 perspective;

void main() {
    vert_tex_coords = tex_coords;
    gl_Position = perspective * vec4(pos, 0.0, 1.0);
}
