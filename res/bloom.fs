#version 140

in vec2 v_tex_coords;
uniform bool horizontal;
const float weight[3] = float[] (0.3125, 0.375, 0.3125);
out vec4 f_color;

void main() {

    vec4 color;
    vec2 offset;

    if (horizontal) {
        offset = vec2(1.2 / sheetSize().x, 0.0);
    } else {
        offset = vec2(0.0, 1.2 / sheetSize().y);
    }

    color  = weight[0] * sheet(v_tex_coords - offset);
    color += weight[1] * sheet(v_tex_coords);
    color += weight[2] * sheet(v_tex_coords + offset);

    f_color = color;
}
