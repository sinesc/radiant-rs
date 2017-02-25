#version 140

uniform mat4 u_view;
uniform mat4 u_model;
uniform vec4 _rd_color;

in vec2 position;
in vec2 offset;
in float rotation;
in vec4 color;
in uint bucket_id;
in uint texture_id;
in vec2 texture_uv;
in uint components;

out vec4 v_color;
out vec2 v_tex_coords;
flat out uint _rd_v_texture_id;
flat out uint _rd_v_bucket_id;
flat out uint _rd_v_components;

void main() {

    // compute vertex positon

    vec2 trans;
    float sin_rotation = sin(rotation);
    float cos_rotation = cos(rotation);
    trans.x = offset.x * cos_rotation - offset.y * sin_rotation;
    trans.y = offset.x * sin_rotation + offset.y * cos_rotation;

    // apply global per sprite matrix (model)

    vec4 model_transformation = u_model * vec4(trans, 0.0, 1.0);

    gl_Position = u_view * vec4(vec3(position, 0.0) + vec3(model_transformation), 1.0);

    // pass along to fragment shader

    v_color = color * _rd_color;
    v_tex_coords = texture_uv;
    _rd_v_texture_id = texture_id;
    _rd_v_bucket_id = bucket_id;
    _rd_v_components = components;
}
