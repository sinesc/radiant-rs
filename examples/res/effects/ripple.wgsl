// Ripple effect — texture fragment shader (for postprocessors / renderer.fill())
// Preamble provides: sheet(), sheetSize(), sheetComponent(), TextureFragmentInput

const PI: f32 = 3.1415926538;
const INTENSITY: f32 = 0.8;
const RANGE: f32 = 0.16;

@fragment
fn main(input: TextureFragmentInput) -> @location(0) vec4<f32> {
    let base = sin(input.v_tex_coords.x) + sin(input.v_tex_coords.y);
    let rand = fract(base * 10000.0);

    let radius = RANGE * sqrt(rand);
    let angle = 2.0 * PI * rand;
    let offset = vec2<f32>(radius * sin(angle), radius * cos(angle));

    return input.v_color * (
          sheet(input.v_tex_coords + offset)         * INTENSITY * 0.05
        + sheet(input.v_tex_coords + offset * 0.8)   * INTENSITY * 0.15
        + sheet(input.v_tex_coords + offset * 0.6)   * INTENSITY * 0.3
        + sheet(input.v_tex_coords + offset * 0.4)   * INTENSITY * 0.5
        + sheet(input.v_tex_coords)                  * (1.0 - INTENSITY)
    );
}
