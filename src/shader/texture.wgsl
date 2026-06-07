// Texture (rectangle) vertex shader - WGSL
// Uses the same Vertex layout as the sprite shader; only location 0 and 6 are read.

@group(0) @binding(0)
var<uniform> texture_uniforms: TextureUniforms;

struct TextureVertex {
    @location(0) position: vec2<f32>,
    @location(6) texture_uv: vec2<f32>,
}

struct TextureVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
}

struct TextureUniforms {
    u_view: mat4x4<f32>,
    u_model: mat4x4<f32>,
    _rd_color: vec4<f32>,
    _rd_offset: vec2<f32>,
    _rd_dimensions: vec2<f32>,
    _rd_flags: vec4<f32>,
}

@vertex
fn main(input: TextureVertex) -> TextureVertexOutput {
    var output: TextureVertexOutput;

    let model_transformation = texture_uniforms.u_model * vec4<f32>(input.position * texture_uniforms._rd_dimensions, 0.0, 1.0);
    output.position = texture_uniforms.u_view * vec4<f32>(vec3<f32>(texture_uniforms._rd_offset, 0.0) + model_transformation.xyz, 1.0);
    output.v_tex_coords = input.texture_uv;
    output.v_color = texture_uniforms._rd_color;

    return output;
}
