use core::{Texture, Renderer, RenderContext, Program, BlendMode, Postprocessor, Color};

/// A basic postprocessor that applies a Program to the given input once.
pub struct Basic {
    source      : Texture,
    program     : Program,
    dimensions  : (f32, f32),
}

impl Postprocessor for Basic {
    fn target(self: &mut Self) -> &Texture {
        &self.source
    }
    fn draw(self: &mut Self, renderer: &Renderer, blendmode: BlendMode) {
        // Simply draws the given source data to the current target using our custom shader program.
        renderer.draw_rect((0., 0.), self.dimensions, blendmode, Some(&self.program), None);
    }
}

impl Basic {
    /// Creates a new instance. Takes the name of the program uniform that is to receive the input sampler2D.
    pub fn new(context: &RenderContext, program: Program, texture_uniform_name: &str) -> Self {

        let (width, height) = context.display().dimensions();

        let mut result = Basic {
            source      : Texture::new(&context, width, height),
            dimensions  : (width as f32, height as f32),
            program     : program,
        };

        result.source.clear(Color::black());
        result.program.set_uniform(texture_uniform_name, &result.source);
        result
    }
}
