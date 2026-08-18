// Separable Gaussian blur — texture fragment shader (preamble style)
// texture_uniforms._rd_flags.x: 1.0 = horizontal pass, 0.0 = vertical pass

const W0: f32 = 0.3125;
const W1: f32 = 0.375;
const W2: f32 = 0.3125;

@fragment
fn main(input: TextureFragmentInput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(sheetSize());
    let horizontal = texture_uniforms._rd_flags.x > 0.5;

    var offset: vec2<f32>;
    if horizontal {
        offset = vec2<f32>(1.2 / tex_size.x, 0.0);
    } else {
        offset = vec2<f32>(0.0, 1.2 / tex_size.y);
    }

    let s0 = input.v_tex_coords - offset;
    let s2 = input.v_tex_coords + offset;

    var color = W1 * sheet(input.v_tex_coords);
    if s0.x > offset.x && s0.y > offset.y {
        color += W0 * sheet(s0);
    }
    if s2.x < 1.0 - offset.x && s2.y < 1.0 - offset.y {
        color += W2 * sheet(s2);
    }

    return color;
}
