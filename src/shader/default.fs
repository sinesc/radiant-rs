#version 140

uniform sampler2DArray tex0;
uniform sampler2DArray tex1;
uniform sampler2DArray tex2;
uniform sampler2DArray tex3;
uniform sampler2DArray tex4;

in vec2 v_tex_coords;
in vec4 v_color;

flat in uint v_texture_id;
flat in uint v_bucket_id;

out vec4 f_color;

void main() {
    // todo there's probably a much better way :)
    if (v_bucket_id == 0u) {
        f_color = texture(tex0, vec3(v_tex_coords, float(v_texture_id))) * (v_color / 255);
    } else if (v_bucket_id == 1u) {
        f_color = texture(tex1, vec3(v_tex_coords, float(v_texture_id))) * (v_color / 255);
    } else if (v_bucket_id == 2u) {
        f_color = texture(tex2, vec3(v_tex_coords, float(v_texture_id))) * (v_color / 255);
    } else if (v_bucket_id == 3u) {
        f_color = texture(tex3, vec3(v_tex_coords, float(v_texture_id))) * (v_color / 255);
    } else if (v_bucket_id == 4u) {
        f_color = texture(tex4, vec3(v_tex_coords, float(v_texture_id))) * (v_color / 255);
    }
}
