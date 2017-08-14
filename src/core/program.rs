use prelude::*;
use core::{self, rendercontext, RenderContext, Display, AsUniform, UniformList, Color};
use maths::{Mat4};
use backends::glium as backend;

const SPRITE_INC: &'static str = include_str!("../shader/sprite.inc.fs");
const TEXTURE_INC: &'static str = include_str!("../shader/texture.inc.fs");
const SPRITE_VS: &'static str = include_str!("../shader/sprite.vs");
const TEXTURE_VS: &'static str = include_str!("../shader/texture.vs");

/// A shader program and its uniforms.
///
/// Cloning a program creates a new program, referencing the internal shaders
/// of the source program but using its own copy of the uniforms.
#[derive(Clone)]
pub struct Program {
    pub uniforms: UniformList,
    sprite_program: Arc<backend::Program>,
    texture_program: Arc<backend::Program>,
}

impl Program {
    /// Creates a program from a fragment shader file.
    pub fn from_file(context: &RenderContext, file: &str) -> core::Result<Self> {
        use std::io::Read;
        let mut source = String::new();
        let mut f = File::open(file)?;
        f.read_to_string(&mut source)?;
        Program::from_string(context, &source)
    }
    /// Creates a program from a fragment shader string.
    pub fn from_string(context: &RenderContext, source: &str) -> core::Result<Self> {
        let context = rendercontext::lock(context);
        create(&context.display, source)
    }
    /// Sets a uniform value by name.
    pub fn set_uniform<T>(self: &mut Self, name: &str, value: &T) where T: AsUniform {
        self.uniforms.insert(name, value.as_uniform());
    }
    /// Removes a uniform value by name.
    pub fn remove_uniform<T>(self: &mut Self, name: &str) -> bool {
        self.uniforms.remove(name)
    }
}

/// Creates a new program. Used in rendercontext creation when the full context is not yet available.
pub fn create(display: &Display, source: &str) -> core::Result<Program> {
    let sprite_fs = insert_template(source, SPRITE_INC);
    let texture_fs = insert_template(source, TEXTURE_INC);
    let display_handle = &display.handle();
    let dimensions = display.dimensions();
    let mut uniforms = UniformList::new();
    uniforms.insert("u_view", Mat4::viewport(dimensions.0 as f32, dimensions.1 as f32).as_uniform());
    uniforms.insert("u_model", Mat4::<f32>::identity().as_uniform());
    uniforms.insert("_rd_color", Color::white().as_uniform());
    Ok(Program {
        uniforms: uniforms,
        sprite_program: Arc::new(backend::Program::new(display_handle, SPRITE_VS, &sprite_fs)?),
        texture_program: Arc::new(backend::Program::new(display_handle, TEXTURE_VS, &texture_fs)?),
    })
}

/// Private accessor to the sprite fragement shader program.
pub fn sprite(program: &Program) -> &backend::Program {
    &program.sprite_program
}

/// Private accessor to the texture fragement shader program.
pub fn texture(program: &Program) -> &backend::Program {
    &program.texture_program
}

/// Inserts program boilterplate code into the shader source.
fn insert_template(source: &str, template: &str) -> String {
    let mut result = String::new();
    let mut lines = source.lines();
    let mut inserted = false;
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push_str("\n");
        if line.starts_with("#") {
            result.push_str(template);
            inserted = true;
        }
    }
    assert!(inserted, "Program is missing a version specifier.");
    result
}
