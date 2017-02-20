uniform sampler2D _rd_tex;
uniform sampler2DArray _rd_tex1;
uniform sampler2DArray _rd_tex2;
uniform sampler2DArray _rd_tex3;
uniform sampler2DArray _rd_tex4;
uniform sampler2DArray _rd_tex5;
uniform uint _rd_comp;

flat in uint _rd_v_texture_id;
flat in uint _rd_v_bucket_id;
flat in uint _rd_v_components;

vec2 sheetSize() {
    if (_rd_v_bucket_id == 0u) {
        return textureSize(_rd_tex, 0);
    } else if (_rd_v_bucket_id == 1u) {
        return textureSize(_rd_tex1, 0).xy;
    } else if (_rd_v_bucket_id == 2u) {
        return textureSize(_rd_tex2, 0).xy;
    } else if (_rd_v_bucket_id == 3u) {
        return textureSize(_rd_tex3, 0).xy;
    } else if (_rd_v_bucket_id == 4u) {
        return textureSize(_rd_tex4, 0).xy;
    } else /*if (_rd_v_bucket_id == 5u)*/ {
        return textureSize(_rd_tex5, 0).xy;
    }
}

vec4 sheetComponent(in vec2 texture_coords, in uint component) {
    if (_rd_v_bucket_id == 0u) {
        return texture(_rd_tex, texture_coords).rrrr;
    } else if (component >= _rd_v_components) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    } else if (_rd_v_bucket_id == 1u) {
        return texture(_rd_tex1, vec3(texture_coords, float(_rd_v_texture_id + component)));
    } else if (_rd_v_bucket_id == 2u) {
        return texture(_rd_tex2, vec3(texture_coords, float(_rd_v_texture_id + component)));
    } else if (_rd_v_bucket_id == 3u) {
        return texture(_rd_tex3, vec3(texture_coords, float(_rd_v_texture_id + component)));
    } else if (_rd_v_bucket_id == 4u) {
        return texture(_rd_tex4, vec3(texture_coords, float(_rd_v_texture_id + component)));
    } else /*if (_rd_v_bucket_id == 5u)*/ {
        return texture(_rd_tex5, vec3(texture_coords, float(_rd_v_texture_id + component)));
    }
}

vec4 sheet(in vec2 texture_coords) {
    return sheetComponent(texture_coords, _rd_comp);
}

vec4 sheetOffset(in vec2 texture_coords, in ivec2 offset) {
    if (_rd_v_bucket_id == 0u) {
        return textureOffset(_rd_tex, texture_coords, offset).rrrr;
    } else if (_rd_comp >= _rd_v_components) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    } else if (_rd_v_bucket_id == 1u) {
        return textureOffset(_rd_tex1, vec3(texture_coords, float(_rd_v_texture_id + _rd_comp)), offset);
    } else if (_rd_v_bucket_id == 2u) {
        return textureOffset(_rd_tex2, vec3(texture_coords, float(_rd_v_texture_id + _rd_comp)), offset);
    } else if (_rd_v_bucket_id == 3u) {
        return textureOffset(_rd_tex3, vec3(texture_coords, float(_rd_v_texture_id + _rd_comp)), offset);
    } else if (_rd_v_bucket_id == 4u) {
        return textureOffset(_rd_tex4, vec3(texture_coords, float(_rd_v_texture_id + _rd_comp)), offset);
    } else /*if (_rd_v_bucket_id == 5u)*/ {
        return textureOffset(_rd_tex5, vec3(texture_coords, float(_rd_v_texture_id + _rd_comp)), offset);
    }
}
