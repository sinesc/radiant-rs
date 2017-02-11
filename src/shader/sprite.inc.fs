uniform sampler2D _rd_tex;
uniform sampler2DArray _rd_tex1;
uniform sampler2DArray _rd_tex2;
uniform sampler2DArray _rd_tex3;
uniform sampler2DArray _rd_tex4;
uniform sampler2DArray _rd_tex5;

flat in uint rd_v_texture_id;
flat in uint rd_v_bucket_id;

vec2 sheetSize() {
    if (rd_v_bucket_id == 0u) {
        return textureSize(_rd_tex, 0);
    } else if (rd_v_bucket_id == 1u) {
        return textureSize(_rd_tex1, 0).xy;
    } else if (rd_v_bucket_id == 2u) {
        return textureSize(_rd_tex2, 0).xy;
    } else if (rd_v_bucket_id == 3u) {
        return textureSize(_rd_tex3, 0).xy;
    } else if (rd_v_bucket_id == 4u) {
        return textureSize(_rd_tex4, 0).xy;
    } else /*if (rd_v_bucket_id == 5u)*/ {
        return textureSize(_rd_tex5, 0).xy;
    }
}

vec4 sheet(in vec2 texture_coords) {
    if (rd_v_bucket_id == 0u) {
        return texture(_rd_tex, texture_coords).rrrr;
    } else if (rd_v_bucket_id == 1u) {
        return texture(_rd_tex1, vec3(texture_coords, float(rd_v_texture_id)));
    } else if (rd_v_bucket_id == 2u) {
        return texture(_rd_tex2, vec3(texture_coords, float(rd_v_texture_id)));
    } else if (rd_v_bucket_id == 3u) {
        return texture(_rd_tex3, vec3(texture_coords, float(rd_v_texture_id)));
    } else if (rd_v_bucket_id == 4u) {
        return texture(_rd_tex4, vec3(texture_coords, float(rd_v_texture_id)));
    } else /*if (rd_v_bucket_id == 5u)*/ {
        return texture(_rd_tex5, vec3(texture_coords, float(rd_v_texture_id)));
    }
}
