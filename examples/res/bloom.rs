use radiant_rs::{Postprocessor, RenderContext, Renderer, Color, Texture, Program, blendmodes};

pub struct Bloom {
    targets         : [[Texture; 5]; 2],
    blur_program    : Program,
    combine_program : Program,
    dimensions      : (f32, f32),
}

impl Postprocessor for Bloom {

    /// Returns the target where the postprocessor expects the unprocessed input.
    fn target(self: &mut Self) -> &Texture {
        self.targets[0][0].clear(Color::black());
        &self.targets[0][0]
    }

    /// Process received data.
    fn process(self: &mut Self, renderer: &Renderer) {

        // Copy to progressively smaller textures
        for i in 1..self.targets[0].len() {
            renderer.set_target(&self.targets[0][i]);
            renderer.draw_rect((0., 0.), self.dimensions, blendmodes::ALPHA, None, Some(&self.targets[0][i-1]));
        }

        // Apply horizontal blur
        self.blur_program.set_uniform("horizontal", &true);
        for i in 0..self.targets[0].len() {
            renderer.set_target(&self.targets[1][i]);
            renderer.draw_rect((0., 0.), self.dimensions, blendmodes::ALPHA, Some(&self.blur_program), Some(&self.targets[0][i]));
        }

        // Apply vertical blur
        self.blur_program.set_uniform("horizontal", &false);
        for i in 0..self.targets[0].len() {
            renderer.set_target(&self.targets[0][i]);
            renderer.draw_rect((0., 0.), self.dimensions, blendmodes::ALPHA, Some(&self.blur_program), Some(&self.targets[1][i]));
        }
    }

    /// Draw processed input. The renderer has already set the correct target.
    fn draw(self: &mut Self, renderer: &Renderer) {
        renderer.draw_rect((0., 0.), self.dimensions, blendmodes::LIGHTEN, Some(&self.combine_program), None);
    }
}

impl Bloom {
    pub fn new(context: &RenderContext) -> Self {

        let blur_program = Program::from_string(&context, include_str!("bloom.fs")).unwrap();
        let combine_program = Program::from_string(&context, include_str!("combine.fs")).unwrap();
        let display = context.display();
        let (width, height) = display.dimensions();

        let mut result = Bloom {
            blur_program: blur_program,
            combine_program: combine_program,
            dimensions: (width as f32, height as f32),
            targets: [ [
                Texture::new(&context, width / 2, height / 2),
                Texture::new(&context, width / 4, height / 4),
                Texture::new(&context, width / 8, height / 8),
                Texture::new(&context, width / 16, height / 16),
                Texture::new(&context, width / 32, height / 32),
            ], [
                Texture::new(&context, width / 2, height / 2),
                Texture::new(&context, width / 4, height / 4),
                Texture::new(&context, width / 8, height / 8),
                Texture::new(&context, width / 16, height / 16),
                Texture::new(&context, width / 32, height / 32),
            ] ]
        };

        result.combine_program.set_uniform("sample0", &result.targets[0][0]);
        result.combine_program.set_uniform("sample1", &result.targets[0][1]);
        result.combine_program.set_uniform("sample2", &result.targets[0][2]);
        result.combine_program.set_uniform("sample3", &result.targets[0][3]);
        result.combine_program.set_uniform("sample4", &result.targets[0][4]);
        result
    }
}
