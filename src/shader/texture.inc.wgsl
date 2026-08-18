// Texture fragment preamble — automatically prepended to custom texture fragment shaders.
// Provides bindings, TextureFragmentInput, and sheet*() helpers.
// Custom shaders write: @fragment fn main(input: TextureFragmentInput) -> @location(0) vec4<f32>
//
// Custom parameters (via Program::set_uniform):
// - f32/bool values are packed into texture_uniforms._rd_flags.x/.y/.z/.w in the order
//   they were first set (up to 4).
// - Texture values are bound to consecutive texture_2d<f32> bindings starting at
//   @group(0) @binding(3) in the order they were first set (up to 8); declare them in
//   the shader and sample them with _rd_sampler (binding 2).

struct TextureUniforms {
    u_view: mat4x4<f32>,
    u_model: mat4x4<f32>,
    _rd_color: vec4<f32>,
    _rd_offset: vec2<f32>,
    _rd_dimensions: vec2<f32>,
    _rd_flags: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> texture_uniforms: TextureUniforms;
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
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
