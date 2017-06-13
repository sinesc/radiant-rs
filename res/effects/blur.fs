#version 330

in vec2 v_tex_coords;
uniform bool horizontal;
const float weight[3] = float[] (0.3125, 0.375, 0.3125);
out vec4 f_color;

void main() {

    vec2 offset;

    if (horizontal) {
        offset = vec2(1.2 / sheetSize().x, 0.0);
    } else {
        offset = vec2(0.0, 1.2 / sheetSize().y);
    }

    vec2 sample0 = v_tex_coords - offset;
    vec2 sample2 = v_tex_coords + offset;

    vec4 color = weight[1] * sheet(v_tex_coords);

    if (sample0.x > offset.x && sample0.y > offset.y) {
        color += weight[0] * sheet(sample0);
    }

    if (sample2.x < 1.0 - offset.x && sample2.y < 1.0 - offset.y) {
        color += weight[2] * sheet(sample2);
    }

    f_color = clamp(color, 0.0, 1.0);
}
