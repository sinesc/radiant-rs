#version 140

in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

const float PI = 3.1415926538;
const float intensity = 0.8;
const float range = 0.06;

void main() {

    float base = sin(v_tex_coords.x) + sin(v_tex_coords.y);
    float rand = fract(base * 10000.0);

    float radius = range * sqrt(rand);
    float angle = 2.0 * PI * rand;

    vec2 offset = vec2(radius * sin(angle), radius * cos(angle));

    f_color = v_color * (sheet(v_tex_coords + offset) * intensity * 0.05
                + sheet(v_tex_coords + offset * 0.8) * intensity * 0.15
                + sheet(v_tex_coords + offset * 0.6) * intensity * 0.3
                + sheet(v_tex_coords + offset * 0.4) * intensity * 0.5
                + sheet(v_tex_coords) * (1.0 - intensity));
}
