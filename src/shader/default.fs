#version 150

in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = sheet(v_tex_coords) * v_color;
    //f_color = f_color + vec4(0.5, 0.0, 0.0, 0.0);
}
