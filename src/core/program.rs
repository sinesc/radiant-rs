use crate::prelude::*;
use crate::core::{Context, AsUniform, UniformList, Color};
use crate::core::math::*;
use crate::backends::backend;

const SPRITE_INC: &str = include_str!("../shader/sprite.inc.wgsl");
const TEXTURE_INC: &str = include_str!("../shader/texture.inc.wgsl");

/// A shader program and its uniforms.
///
/// Cloning a program creates a new program, referencing the internal shaders
/// of the source program but using its own copy of the uniforms.
#[derive(Clone)]
pub struct Program {
    pub uniforms: UniformList,
    pub(crate) sprite_program: Option<Arc<backend::Program>>,
    pub(crate) texture_program: Arc<backend::Program>,
}

impl Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Program")
            .field("uniforms", &self.uniforms)
            .finish()
    }
}

impl Program {
    /// Creates a program from a fragment shader file.
    pub fn from_file(context: &Context, file: &str) -> crate::core::Result<Self> {
        use std::io::Read;
        let mut source = String::new();
        let mut f = File::open(file)?;
        f.read_to_string(&mut source)?;
        Self::from_string(context, &source)
    }
    /// Creates a program from a WGSL fragment shader string.
    ///
    /// If the shader does not already declare its own uniforms buffer (`@group(0) @binding(0)`),
    /// the engine automatically prepends the appropriate preamble, which provides
    /// `sheet()`, `sheetSize()`, `sheetComponent()`, and the input struct.
    ///
    /// For texture shaders (postprocessors, fills): write `@fragment fn main(input: TextureFragmentInput)`.
    /// For sprite shaders (layers): write `@fragment fn main(input: SpriteFragmentInput)`.
    ///
    /// Custom parameters set via [`set_uniform`](Self::set_uniform) are routed to texture
    /// shaders as follows:
    /// - `f32` and `bool` values are packed into `texture_uniforms._rd_flags` (`.x`, `.y`,
    ///   `.z`, `.w` in the order they were first set, at most 4 values).
    /// - `Texture` values are bound as additional `texture_2d<f32>` inputs in consecutive
    ///   bindings starting at `@group(0) @binding(3)` (at most 8 textures, in the order they
    ///   were first set). Declare them in the shader and sample them with the sampler at
    ///   `@group(0) @binding(2)`.
    ///
    /// Sprite shaders do not support custom parameters.
    pub fn from_string(context: &Context, source: &str) -> crate::core::Result<Self> {
        Self::new(context, source)
    }
    /// Sets a uniform value by name.
    pub fn set_uniform<T>(self: &mut Self, name: &str, value: &T) where T: AsUniform {
        self.uniforms.insert(name, value.as_uniform());
    }
    /// Removes a uniform value by name.
    pub fn remove_uniform<T>(self: &mut Self, name: &str) -> bool {
        self.uniforms.remove(name)
    }
    /// Creates a new program from user-provided WGSL source.
    pub(crate) fn new(context: &Context, source: &str) -> crate::core::Result<Program> {
        let mut uniforms = UniformList::new();
        uniforms.insert("u_view", Mat4::viewport(1.0, 1.0).as_uniform());
        uniforms.insert("u_model", Mat4::<f32>::identity().as_uniform());
        uniforms.insert("_rd_color", Color::WHITE.as_uniform());
        let context = context.lock();
        let backend_context = context.backend_context.as_ref().unwrap();

        // A sprite shader takes SpriteFragmentInput as its fragment input (see from_string).
        let is_sprite = source.contains("SpriteFragmentInput"); // FIXME: add ShaderType::SPRITE/TEXTURE parameter instead of this

        // Prepend preamble unless the shader is self-contained (already declares bindings).
        // Self-contained detection: a shader is self-contained if it declares the
        // uniforms buffer itself (@group(0) @binding(0)). Preamble-style shaders may
        // declare user textures at binding 3+ without being self-contained.
        let combined = if source.contains("@group(0) @binding(0)") {
            source.to_string()
        } else if is_sprite {
            format!("{}\n{}", SPRITE_INC, source)
        } else {
            format!("{}\n{}", TEXTURE_INC, source)
        };

        // Sprite programs draw sprite layers; they also get the built-in default texture
        // program so they can be used with fills (no visible effect there).
        let (sprite_program, texture_program) = if is_sprite {
            let sprite = Arc::new(backend::Program::new_sprite(backend_context, &combined)?);
            let texture = Arc::new(backend::Program::new_default_texture(backend_context)?);
            (Some(sprite), texture)
        } else {
            let texture = Arc::new(backend::Program::new_texture(backend_context, &combined)?);
            (None, texture)
        };

        Ok(Program { uniforms, sprite_program, texture_program })
    }
    /// Creates the built-in default program (no custom effect).
    pub(crate) fn new_default(context: &Context) -> crate::core::Result<Program> {
        let mut uniforms = UniformList::new();
        uniforms.insert("u_view", Mat4::viewport(1.0, 1.0).as_uniform());
        uniforms.insert("u_model", Mat4::<f32>::identity().as_uniform());
        uniforms.insert("_rd_color", Color::WHITE.as_uniform());
        let context = context.lock();
        let backend_context = context.backend_context.as_ref().unwrap();
        let texture_program = Arc::new(backend::Program::new_default_texture(backend_context)?);
        Ok(Program { uniforms, sprite_program: None, texture_program })
    }
}
