uniform sampler2D tex;
uniform sampler2DArray tex1;
uniform sampler2DArray tex2;
uniform sampler2DArray tex3;
uniform sampler2DArray tex4;
uniform sampler2DArray tex5;

flat in uint v_texture_id;
flat in uint v_bucket_id;

vec2 sheetSize() {
    if (v_bucket_id == 0u) {
        return textureSize(tex, 0);
    } else if (v_bucket_id == 1u) {
        return textureSize(tex1, 0).xy;
    } else if (v_bucket_id == 2u) {
        return textureSize(tex2, 0).xy;
    } else if (v_bucket_id == 3u) {
        return textureSize(tex3, 0).xy;
    } else if (v_bucket_id == 4u) {
        return textureSize(tex4, 0).xy;
    } else /*if (v_bucket_id == 5u)*/ {
        return textureSize(tex5, 0).xy;
    }
}

vec4 sheet(in vec2 texture_coords) {
    if (v_bucket_id == 0u) {
        return texture(tex, texture_coords).rrrr;
    } else if (v_bucket_id == 1u) {
        return texture(tex1, vec3(texture_coords, float(v_texture_id)));
    } else if (v_bucket_id == 2u) {
        return texture(tex2, vec3(texture_coords, float(v_texture_id)));
    } else if (v_bucket_id == 3u) {
        return texture(tex3, vec3(texture_coords, float(v_texture_id)));
    } else if (v_bucket_id == 4u) {
        return texture(tex4, vec3(texture_coords, float(v_texture_id)));
    } else /*if (v_bucket_id == 5u)*/ {
        return texture(tex5, vec3(texture_coords, float(v_texture_id)));
    }
}
