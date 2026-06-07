// Texture (rectangle) fragment shader - WGSL
// Always samples the texture; pass a 1x1 white texture when no real texture is needed.

@group(0) @binding(1)
var _rd_tex: texture_2d<f32>;
@group(0) @binding(2)
var _rd_sampler: sampler;

struct TextureFragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
}

fn sheet(texture_coords: vec2<f32>) -> vec4<f32> {
    return textureSample(_rd_tex, _rd_sampler, texture_coords);
}

fn sheetSize() -> vec2<i32> {
    return vec2<i32>(textureDimensions(_rd_tex));
}

fn sheetComponent(texture_coords: vec2<f32>, component: u32) -> vec4<f32> {
    if (component == 0u) {
        return sheet(texture_coords);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

@fragment
fn main(input: TextureFragmentInput) -> @location(0) vec4<f32> {
    return sheet(input.v_tex_coords) * input.v_color;
}
