#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;

void main() {
    mat4 color_correction = mat4(
    // row 1
    0.0, 0.0, 1.0, 0.0,
    // row 2
    0.0, 1.0, 0.0, 0.0,
    // row 3
    1.0, 0.0, 0.0, 0.0,
    // row 4
    0.0, 0.0, 0.0, 1.0);
    color = texture(tex, v_tex_coords) * color_correction;
}
