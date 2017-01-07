#version 140

uniform sampler2D font_cache;
uniform sampler2DArray tex1;
uniform sampler2DArray tex2;
uniform sampler2DArray tex3;
uniform sampler2DArray tex4;
uniform sampler2DArray tex5;
uniform vec4 global_color;

in vec2 v_tex_coords;
in vec4 v_color;

flat in uint v_texture_id;
flat in uint v_bucket_id;

out vec4 f_color;

void main() {
    // todo there's probably a much better way :)

    vec4 color = v_color * global_color;

    if (v_bucket_id == 0u) {
        f_color = texture(font_cache, v_tex_coords).r * color;
    } else if (v_bucket_id == 1u) {
        f_color = texture(tex1, vec3(v_tex_coords, float(v_texture_id))) * color;
    } else if (v_bucket_id == 2u) {
        f_color = texture(tex2, vec3(v_tex_coords, float(v_texture_id))) * color;
    } else if (v_bucket_id == 3u) {
        f_color = texture(tex3, vec3(v_tex_coords, float(v_texture_id))) * color;
    } else if (v_bucket_id == 4u) {
        f_color = texture(tex4, vec3(v_tex_coords, float(v_texture_id))) * color;
    } else if (v_bucket_id == 5u) {
        f_color = texture(tex5, vec3(v_tex_coords, float(v_texture_id))) * color;
    }
}
