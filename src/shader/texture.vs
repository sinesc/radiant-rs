#version 140

uniform mat4 u_view;
uniform mat4 u_model;
uniform vec4 _rd_color;
uniform vec2 _rd_offset;
uniform vec2 _rd_dimensions;

in vec2 position;
in vec2 texture_uv;

out vec4 v_color;
out vec2 v_tex_coords;

void main() {

    vec2 v_pos = _rd_offset + vec2( vec4(position * _rd_dimensions, 0.0, 1.0) * u_model );

    vec4 final = u_view * vec4(v_pos, 0.0, 1.0);

    gl_Position = final;
    v_tex_coords = texture_uv;
    v_color = _rd_color;
}
