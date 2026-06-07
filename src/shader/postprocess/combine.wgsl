// Bloom combine fragment shader (WGSL)
// Combines 5 mip textures; brightness from _rd_flags.x (default 1.0)

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
var sample0: texture_2d<f32>;
@group(0) @binding(2)
var sample1: texture_2d<f32>;
@group(0) @binding(3)
var sample2: texture_2d<f32>;
@group(0) @binding(4)
var sample3: texture_2d<f32>;
@group(0) @binding(5)
var sample4: texture_2d<f32>;
@group(0) @binding(6)
var _rd_sampler: sampler;

struct FragInput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
}

@fragment
fn main(input: FragInput) -> @location(0) vec4<f32> {
    let uv = input.v_tex_coords;
    let t0 = textureSample(sample0, _rd_sampler, uv);
    let t1 = textureSample(sample1, _rd_sampler, uv);
    let t2 = textureSample(sample2, _rd_sampler, uv);
    let t3 = textureSample(sample3, _rd_sampler, uv);
    let t4 = textureSample(sample4, _rd_sampler, uv);
    let brightness = texture_uniforms._rd_flags.y;
    return clamp((t0 + t1 + t2 + t3 + t4) * brightness, vec4<f32>(0.0), vec4<f32>(1.0));
}
