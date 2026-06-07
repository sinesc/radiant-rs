// Separable Gaussian blur fragment shader (WGSL)
// _rd_flags.x: 1.0 = horizontal pass, 0.0 = vertical pass

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

struct FragInput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
}

const W0: f32 = 0.3125;
const W1: f32 = 0.375;
const W2: f32 = 0.3125;

@fragment
fn main(input: FragInput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(_rd_tex));
    let horizontal = texture_uniforms._rd_flags.x > 0.5;

    var offset: vec2<f32>;
    if horizontal {
        offset = vec2<f32>(1.2 / tex_size.x, 0.0);
    } else {
        offset = vec2<f32>(0.0, 1.2 / tex_size.y);
    }

    let s0 = input.v_tex_coords - offset;
    let s2 = input.v_tex_coords + offset;

    var color = W1 * textureSample(_rd_tex, _rd_sampler, input.v_tex_coords);

    if s0.x > offset.x && s0.y > offset.y {
        color += W0 * textureSample(_rd_tex, _rd_sampler, s0);
    }
    if s2.x < 1.0 - offset.x && s2.y < 1.0 - offset.y {
        color += W2 * textureSample(_rd_tex, _rd_sampler, s2);
    }

    return color;
}
