#version 140

uniform mat4 u_view;
uniform vec4 _rd_color;
uniform vec2 _rd_offset;
uniform vec2 _rd_dimensions;

in vec2 position;
in vec2 texture_uv;

out vec4 v_color;
out vec2 v_tex_coords;

void main() {

    int id = gl_VertexID;
    vec2 v_pos;
    vec2 f_pos;

    if (id == 0) {
        v_pos = _rd_offset;
        f_pos = vec2(0.0, 1.0);
    } else if (id == 1) {
        v_pos = vec2(_rd_offset.x, _rd_offset.y + _rd_dimensions.y);
        f_pos = vec2(0.0, 0.0);
    } else if (id == 2) {
        v_pos = vec2(_rd_offset.x + _rd_dimensions.x, _rd_offset.y);
        f_pos = vec2(1.0, 1.0);
    } else if (id == 3) {
        v_pos = vec2(_rd_offset.x + _rd_dimensions.x, _rd_offset.y + _rd_dimensions.y);
        f_pos = vec2(1.0, 0.0);
    }

    gl_Position = u_view * vec4(v_pos, 0.0, 1.0);
    v_tex_coords = f_pos;
    v_color = _rd_color;
}
