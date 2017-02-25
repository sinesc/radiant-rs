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

    vec4 model_transformation = u_model * vec4(position * _rd_dimensions, 0.0, 1.0);

    gl_Position = u_view * vec4(vec3(_rd_offset, 0.0) + vec3(model_transformation), 1.0);

    // pass along to fragment shade

    v_tex_coords = texture_uv;
    v_color = _rd_color;
}
