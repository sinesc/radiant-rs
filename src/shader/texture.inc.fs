uniform sampler2D tex;

vec2 sheetSize() {
    return textureSize(tex, 0).xy;
}

vec4 sheet(in vec2 texture_coords) {
    return texture(tex, texture_coords);
}
