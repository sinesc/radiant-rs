uniform sampler2D _rd_tex;
uniform bool _rd_has_tex;

ivec2 sheetSize() {
    return textureSize(_rd_tex, 0).xy;
}

vec4 sheet(in vec2 texture_coords) {
    if (_rd_has_tex) {
        return texture(_rd_tex, texture_coords);
    } else {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }
}

vec4 sheetComponent(in vec2 texture_coords, in uint component) {
    if (component == 0u) {
        if (_rd_has_tex) {
            return texture(_rd_tex, texture_coords);
        } else {
            return vec4(1.0, 1.0, 1.0, 1.0);
        }
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}
/*
vec4 sheetOffset(in vec2 texture_coords, in ivec2 offset) {
    if (_rd_has_tex) {
        return textureOffset(_rd_tex, texture_coords, offset);
    } else {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }
}
*/