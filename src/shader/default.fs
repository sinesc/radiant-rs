#version 140

uniform vec4 global_color;

in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

void main() {

    vec4 color = v_color * global_color;

    f_color = sheet(v_tex_coords) * color;
    //f_color = f_color + vec4(0.5, 0.0, 0.0, 0.0);
}
