#version 140

uniform mat4 view_matrix;
uniform mat4 model_matrix;

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

    // compute vertex positon

    vec2 trans;
    float sin_rotation = sin(rotation);
    float cos_rotation = cos(rotation);
    trans.x = offset.x * cos_rotation - offset.y * sin_rotation;
    trans.y = offset.x * sin_rotation + offset.y * cos_rotation;

    // apply global per sprite matrix (model)

    vec4 final_trans = model_matrix * vec4(trans, 0.0, 1.0);

    gl_Position = view_matrix * vec4(position + vec2(final_trans), 0.0, 1.0);

    // pass along to fragment shader

    v_color = color;
    v_bucket_id = bucket_id;
    v_tex_coords = texture_uv;
    v_texture_id = texture_id;
}
