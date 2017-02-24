use core::{Texture, Renderer, RenderContext, Program, BlendMode, Postprocessor, Color};
use maths::{Rect, Point2};

/// A basic postprocessor that applies a Program to the given input once.
///
/// Postprocessors are used with [`Renderer::postprocess()`](../struct.Renderer.html#method.postprocess).
/// The associated type for this postprocessor is `BlendMode` and is expected as second argument
/// to the `postprocess()` method
///
/// ```
/// // Load a shader progam.
/// let my_program = Program::from_string(&rendercontext, include_str!("my_shader.fs")).unwrap();
///
/// // Create the postprocessor with the program.
/// let my_postprocessor = postprocessors::Basic::new(&rendercontext, my_program);
///
/// // ... in your renderloop...
/// renderer.postprocess(&my_postprocessor, &blendmodes::ALPHA, || {
///     renderer.clear(Color::black());
///     renderer.draw_layer(&your_layer, 0);
/// });
/// ```
pub struct Basic {
    source      : Texture,
    program     : Program,
    dimensions  : Point2,
}

impl Postprocessor for Basic {
    /// The Basic postprocessor accepts a blendmode as argument to `Renderer::postprocess()`.
    type T = BlendMode;
    fn target(self: &Self) -> &Texture {
        &self.source
    }
    fn draw(self: &Self, renderer: &Renderer, blendmode: &Self::T) {
        renderer.rect(Rect(Point2(0., 0.), self.dimensions)).blendmode(*blendmode).program(&self.program).texture(&self.source).draw();
    }
}

impl Basic {
    /// Creates a new instance. The shader can use `sheet*()` to access the input texture.
    pub fn new(context: &RenderContext, program: Program) -> Self {

        let Point2(width, height) = context.display().dimensions();

        let result = Basic {
            source      : Texture::new(&context, width, height),
            dimensions  : Point2(width as f32, height as f32),
            program     : program,
        };

        result.source.clear(Color::black());
        result
    }
}
