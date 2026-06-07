// Sprite vertex shader - WGSL

@group(0) @binding(0)
var<uniform> sprite_uniforms: SpriteUniforms;

struct SpriteVertex {
    @location(0) position: vec2<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) color: vec4<f32>,
    @location(4) bucket_id: u32,
    @location(5) texture_id: u32,
    @location(6) texture_uv: vec2<f32>,
    @location(7) components: u32,
}

struct SpriteVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_color: vec4<f32>,
    @location(1) v_tex_coords: vec2<f32>,
    @location(2) _rd_v_texture_id: u32,
    @location(3) _rd_v_bucket_id: u32,
    @location(4) _rd_v_components: u32,
}

struct SpriteUniforms {
    u_view: mat4x4<f32>,
    u_model: mat4x4<f32>,
    _rd_color: vec4<f32>,
}

@vertex
fn main(input: SpriteVertex) -> SpriteVertexOutput {
    var output: SpriteVertexOutput;

    // compute vertex position
    let sin_rotation = sin(input.rotation);
    let cos_rotation = cos(input.rotation);
    var trans = vec2<f32>(
        input.offset.x * cos_rotation - input.offset.y * sin_rotation,
        input.offset.x * sin_rotation + input.offset.y * cos_rotation,
    );

    // apply global per sprite matrix (model)
    let model_transformation = sprite_uniforms.u_model * vec4<f32>(trans, 0.0, 1.0);

    output.position = sprite_uniforms.u_view * vec4<f32>(vec3<f32>(input.position, 0.0) + model_transformation.xyz, 1.0);

    // pass along to fragment shader
    output.v_color = input.color * sprite_uniforms._rd_color;
    output.v_tex_coords = input.texture_uv;
    output._rd_v_texture_id = input.texture_id;
    output._rd_v_bucket_id = input.bucket_id;
    output._rd_v_components = input.components;

    return output;
}
