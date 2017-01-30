#version 140

uniform mat4 view_matrix;
uniform vec2 offset;
uniform vec2 size;

in vec2 position;
in vec2 texture_uv;

out vec2 v_tex_coords;

void main() {

    int id = gl_VertexID;
    vec2 v_pos;
    vec2 f_pos;

    if (id == 0) {
        v_pos = offset;
        f_pos = vec2(0.0, 1.0);
    } else if (id == 1) {
        v_pos = vec2(offset.x, offset.y + size.y);
        f_pos = vec2(0.0, 0.0);
    } else if (id == 2) {
        v_pos = vec2(offset.x + size.x, offset.y);
        f_pos = vec2(1.0, 1.0);
    } else if (id == 3) {
        v_pos = vec2(offset.x + size.x, offset.y + size.y);
        f_pos = vec2(1.0, 0.0);
    }

    gl_Position = view_matrix * vec4(v_pos, 0.0, 1.0);
    v_tex_coords = f_pos;
}
