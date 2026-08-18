// Bloom combine — texture fragment shader (preamble style)
// Combines 5 progressively scaled textures, provided as the program's texture
// uniforms sample0..sample4 (bound to @group(0) @binding(3)..=7).
// Brightness is passed via program.set_uniform("brightness", &value) — the first
// scalar uniform set, so it lands in texture_uniforms._rd_flags.x.

@group(0) @binding(3) var sample0: texture_2d<f32>;
@group(0) @binding(4) var sample1: texture_2d<f32>;
@group(0) @binding(5) var sample2: texture_2d<f32>;
@group(0) @binding(6) var sample3: texture_2d<f32>;
@group(0) @binding(7) var sample4: texture_2d<f32>;

@fragment
fn main(input: TextureFragmentInput) -> @location(0) vec4<f32> {
    let uv = input.v_tex_coords;
    let t0 = textureSample(sample0, _rd_sampler, uv);
    let t1 = textureSample(sample1, _rd_sampler, uv);
    let t2 = textureSample(sample2, _rd_sampler, uv);
    let t3 = textureSample(sample3, _rd_sampler, uv);
    let t4 = textureSample(sample4, _rd_sampler, uv);
    let brightness = texture_uniforms._rd_flags.x;
    return clamp((t0 + t1 + t2 + t3 + t4) * brightness, vec4<f32>(0.0), vec4<f32>(1.0));
}
