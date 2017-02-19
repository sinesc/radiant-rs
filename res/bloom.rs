use radiant_rs::{Postprocessor, RenderContext, Renderer, Color, Texture, Program, BlendMode, blendmodes};
use std::sync::Mutex;

pub struct Bloom {
    targets         : [[Texture; 5]; 2],
    blur_program    : Mutex<Program>,
    combine_program : Mutex<Program>,
    dimensions      : (f32, f32),
    iterations      : u32,
    iter_blend      : BlendMode,
}

impl Postprocessor for Bloom {
    type T = BlendMode;

    /// Returns the target where the postprocessor expects the unprocessed input.
    fn target(self: &Self) -> &Texture {
        &self.targets[0][0]
    }

    /// Process received data.
    fn process(self: &Self, renderer: &Renderer, _: &Self::T) {
        use std::ops::DerefMut;

        // Copy to progressively smaller textures
        for i in 1..self.targets[0].len() {
            renderer.set_target(&self.targets[0][i]);
            renderer.draw_rect((0., 0.), self.dimensions, blendmodes::COPY, None, Some(&self.targets[0][i-1]));
        }

        let mut blur = self.blur_program.lock().unwrap();
        let blur = blur.deref_mut();

        for _ in 0..self.iterations {

            // Apply horizontal blur
            blur.set_uniform("horizontal", &true);
            for i in 0..self.targets[1].len() {
                renderer.set_target(&self.targets[1][i]);
                renderer.draw_rect((0., 0.), self.dimensions, self.iter_blend, Some(&blur), Some(&self.targets[0][i]));
            }

            // Apply vertical blur
            blur.set_uniform("horizontal", &false);
            for i in 0..self.targets[0].len() {
                renderer.set_target(&self.targets[0][i]);
                renderer.draw_rect((0., 0.), self.dimensions, self.iter_blend, Some(&blur), Some(&self.targets[1][i]));
            }
        }
    }

    /// Draw processed input. The renderer has already set the correct target.
    fn draw(self: &Self, renderer: &Renderer, blendmode: &Self::T) {
        use std::ops::DerefMut;
        let mut combine = self.combine_program.lock().unwrap();
        let combine = combine.deref_mut();
        renderer.draw_rect((0., 0.), self.dimensions, *blendmode, Some(combine), None);
        self.targets[0][0].clear(Color::transparent());
    }
}

impl Bloom {
    pub fn new(context: &RenderContext, iterations: u32, iter_blend: BlendMode) -> Self {
        use std::ops::DerefMut;

        let blur_program = Program::from_string(&context, include_str!("blur.fs")).unwrap();
        let combine_program = Program::from_string(&context, include_str!("combine.fs")).unwrap();
        let display = context.display();
        let (width, height) = display.dimensions();

        let result = Bloom {
            blur_program    : Mutex::new(blur_program),
            combine_program : Mutex::new(combine_program),
            dimensions      : (width as f32, height as f32),
            iterations      : iterations,
            iter_blend      : iter_blend,
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

        {
            let mut combine = result.combine_program.lock().unwrap();
            let combine = combine.deref_mut();
            combine.set_uniform("sample0", &result.targets[0][0]);
            combine.set_uniform("sample1", &result.targets[0][1]);
            combine.set_uniform("sample2", &result.targets[0][2]);
            combine.set_uniform("sample3", &result.targets[0][3]);
            combine.set_uniform("sample4", &result.targets[0][4]);
        }

        result
    }
}
