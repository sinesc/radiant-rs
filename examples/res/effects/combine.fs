#version 330

in vec2 v_tex_coords;
uniform sampler2D sample0;
uniform sampler2D sample1;
uniform sampler2D sample2;
uniform sampler2D sample3;
uniform sampler2D sample4;
uniform float brightness;
out vec4 f_color;

void main(void) {
    vec4 t0 = texture2D(sample0, v_tex_coords);
    vec4 t1 = texture2D(sample1, v_tex_coords);
    vec4 t2 = texture2D(sample2, v_tex_coords);
    vec4 t3 = texture2D(sample3, v_tex_coords);
    vec4 t4 = texture2D(sample4, v_tex_coords);
    f_color = clamp((t0 + t1 + t2 + t3 + t4) * brightness, 0.0, 1.0);
}
