#version 140

uniform mat4 matrix;

in vec2 position;
in vec2 offset;
in float rotation;
in vec4 color;
in uint bucket_id;
in uint texture_id;
in vec2 texture_uv;

out vec2 v_tex_coords;
out vec4 v_color;
flat out uint v_texture_id;
flat out uint v_bucket_id;

void main() {

    // compute final vertex positon

    vec2 trans;
    float sin_rotation = sin(rotation);
    float cos_rotation = cos(rotation);
    trans.x = offset.x * cos_rotation - offset.y * sin_rotation;
    trans.y = offset.x * sin_rotation + offset.y * cos_rotation;

    gl_Position = matrix * vec4(position + trans, 0.0, 1.0);

    // compute fragment shader information

    /*if (gl_VertexID % 4 == 0) {
        v_tex_coords = vec2(0.0, 1.0);
    } else if (gl_VertexID % 4 == 1) {
        v_tex_coords = vec2(1.0, 1.0);
    } else if (gl_VertexID % 4 == 2) {
        v_tex_coords = vec2(0.0, 0.0);
    } else {
        v_tex_coords = vec2(1.0, 0.0);
    }*/

    v_color = color;
    v_bucket_id = bucket_id;
    v_tex_coords = texture_uv;
    v_texture_id = texture_id;
}
