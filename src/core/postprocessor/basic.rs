use core::{Texture, Renderer, RenderContext, Program, BlendMode, Postprocessor, Color};

/// A basic postprocessor that applies a Program to the given input once.
pub struct Basic {
    source      : Texture,
    program     : Program,
    dimensions  : (f32, f32),
}

impl Postprocessor for Basic {
    type T = BlendMode;
    fn target(self: &Self) -> &Texture {
        &self.source
    }
    fn draw(self: &Self, renderer: &Renderer, blendmode: &Self::T) {
        // Simply draws the given source data to the current target using our custom shader program.
        renderer.draw_rect((0., 0.), self.dimensions, *blendmode, Some(&self.program), Some(&self.source));
    }
}

impl Basic {
    /// Creates a new instance. The shader can use sheet*() to access the input texture.
    pub fn new(context: &RenderContext, program: Program) -> Self {

        let (width, height) = context.display().dimensions();

        let result = Basic {
            source      : Texture::new(&context, width, height),
            dimensions  : (width as f32, height as f32),
            program     : program,
        };

        result.source.clear(Color::black());
        result
    }
}
