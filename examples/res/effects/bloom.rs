use radiant_rs::*;
use std::sync::Mutex;

pub struct Bloom {
    targets         : [[Texture; 5]; 2],
    blur_program    : Mutex<Program>,
    combine_program : Mutex<Program>,
    iterations      : u32,
    spread          : u8,
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
        for i in 1..self.spread as usize {
            renderer.render_to(&self.targets[0][i], || {
                renderer.copy_from(&self.targets[0][i-1], TextureFilter::Linear);
            });
        }

        let mut blur = self.blur_program.lock().unwrap();
        let blur = blur.deref_mut();

        for _ in 0..self.iterations {

            // Apply horizontal blur
            blur.set_uniform("horizontal", &true);
            for i in 0..self.spread as usize {
                renderer.render_to(&self.targets[1][i], || {
                    renderer.fill().blendmode(blendmodes::ALPHA).program(&blur).texture(&self.targets[0][i]).draw();
                });
            }

            // Apply vertical blur
            blur.set_uniform("horizontal", &false);
            for i in 0..self.spread as usize {
                renderer.render_to(&self.targets[0][i], || {
                    renderer.fill().blendmode(blendmodes::ALPHA).program(&blur).texture(&self.targets[1][i]).draw();
                });
            }
        }
    }

    /// Draw processed input. The renderer has already set the correct target.
    fn draw(self: &Self, renderer: &Renderer, blendmode: &Self::T) {
        use std::ops::DerefMut;
        let mut combine = self.combine_program.lock().unwrap();
        let combine = combine.deref_mut();
        renderer.fill().blendmode(*blendmode).program(&combine).draw();
        self.targets[0][0].clear(Color::TRANSPARENT);
    }
}

impl Bloom {
    pub fn new(context: &RenderContext, dimensions: (u32, u32), iterations: u32, spread: u8, brightness: f32) -> Self {
        use std::cmp::min;

        let blur_program = Program::from_string(&context, include_str!("blur.fs")).unwrap();
        let mut combine_program = Program::from_string(&context, include_str!("combine.fs")).unwrap();
        let (width, height) = dimensions;
        let builder = Texture::builder(context).format(TextureFormat::F16F16F16F16);

        let targets = [ [
            builder.clone().dimensions((width / 2, height / 2)).build().unwrap(),
            builder.clone().dimensions((width / 4, height / 4)).build().unwrap(),
            builder.clone().dimensions((width / 8, height / 8)).build().unwrap(),
            builder.clone().dimensions((width / 16, height / 16)).build().unwrap(),
            builder.clone().dimensions((width / 32, height / 32)).build().unwrap(),
        ], [
            builder.clone().dimensions((width / 2, height / 2)).build().unwrap(),
            builder.clone().dimensions((width / 4, height / 4)).build().unwrap(),
            builder.clone().dimensions((width / 8, height / 8)).build().unwrap(),
            builder.clone().dimensions((width / 16, height / 16)).build().unwrap(),
            builder.clone().dimensions((width / 32, height / 32)).build().unwrap(),
        ] ];

        let spread = min(targets[0].len() as u8, spread);

        combine_program.set_uniform("brightness", &(brightness / spread as f32));
        combine_program.set_uniform("sample0", &targets[0][0]);
        combine_program.set_uniform("sample1", &targets[0][1]);
        combine_program.set_uniform("sample2", &targets[0][2]);
        combine_program.set_uniform("sample3", &targets[0][3]);
        combine_program.set_uniform("sample4", &targets[0][4]);

        Bloom {
            blur_program    : Mutex::new(blur_program),
            combine_program : Mutex::new(combine_program),
            iterations      : iterations,
            targets         : targets,
            spread          : spread,
        }
    }
}
