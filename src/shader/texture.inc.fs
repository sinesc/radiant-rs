uniform sampler2D _rd_tex;

vec2 sheetSize() {
    return textureSize(_rd_tex, 0).xy;
}

vec4 sheet(in vec2 texture_coords) {
    return texture(_rd_tex, texture_coords);
}

vec4 sheetOffset(in vec2 texture_coords, in ivec2 offset) {
    return textureOffset(_rd_tex, texture_coords, offset);
}
