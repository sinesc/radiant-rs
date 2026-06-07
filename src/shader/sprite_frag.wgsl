// Sprite fragment shader - WGSL
// This is the combined fragment shader (includes the sprite texture sampling logic)

@group(0) @binding(1)
var _rd_tex: texture_2d<f32>;
@group(0) @binding(2)
var _rd_sampler: sampler;

@group(0) @binding(3)
var _rd_tex1: texture_2d_array<f32>;
@group(0) @binding(4)
var _rd_tex2: texture_2d_array<f32>;
@group(0) @binding(5)
var _rd_tex3: texture_2d_array<f32>;
@group(0) @binding(6)
var _rd_tex4: texture_2d_array<f32>;
@group(0) @binding(7)
var _rd_tex5: texture_2d_array<f32>;

@group(0) @binding(8)
var<uniform> _rd_comp: u32;

struct SpriteFragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
    @location(2) _rd_v_texture_id: u32,
    @location(3) _rd_v_bucket_id: u32,
    @location(4) _rd_v_components: u32,
}

fn sheetSize(input: SpriteFragmentInput) -> vec2<i32> {
    if (input._rd_v_bucket_id == 0u) {
        return vec2<i32>(textureDimensions(_rd_tex));
    } else if (input._rd_v_bucket_id == 1u) {
        return vec2<i32>(textureDimensions(_rd_tex1).xy);
    } else if (input._rd_v_bucket_id == 2u) {
        return vec2<i32>(textureDimensions(_rd_tex2).xy);
    } else if (input._rd_v_bucket_id == 3u) {
        return vec2<i32>(textureDimensions(_rd_tex3).xy);
    } else if (input._rd_v_bucket_id == 4u) {
        return vec2<i32>(textureDimensions(_rd_tex4).xy);
    } else {
        return vec2<i32>(textureDimensions(_rd_tex5).xy);
    }
}

fn sheetComponent(input: SpriteFragmentInput, texture_coords: vec2<f32>, component: u32) -> vec4<f32> {
    if (input._rd_v_bucket_id == 0u) {
        return textureSample(_rd_tex, _rd_sampler, texture_coords).rrrr;
    } else if (component >= input._rd_v_components) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else if (input._rd_v_bucket_id == 1u) {
        return textureSample(_rd_tex1, _rd_sampler, texture_coords, u32(input._rd_v_texture_id + component));
    } else if (input._rd_v_bucket_id == 2u) {
        return textureSample(_rd_tex2, _rd_sampler, texture_coords, u32(input._rd_v_texture_id + component));
    } else if (input._rd_v_bucket_id == 3u) {
        return textureSample(_rd_tex3, _rd_sampler, texture_coords, u32(input._rd_v_texture_id + component));
    } else if (input._rd_v_bucket_id == 4u) {
        return textureSample(_rd_tex4, _rd_sampler, texture_coords, u32(input._rd_v_texture_id + component));
    } else {
        return textureSample(_rd_tex5, _rd_sampler, texture_coords, u32(input._rd_v_texture_id + component));
    }
}

fn sheet(input: SpriteFragmentInput, texture_coords: vec2<f32>) -> vec4<f32> {
    return sheetComponent(input, texture_coords, _rd_comp);
}

// User fragment shader entry point (called by the default or custom shader)
@fragment
fn main(input: SpriteFragmentInput) -> @location(0) vec4<f32> {
    return sheet(input, input.v_tex_coords) * input.v_color;
}
