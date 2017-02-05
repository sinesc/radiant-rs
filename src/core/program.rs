use prelude::*;
use core::{layer, Layer, rendercontext, RenderContext, display, Display};
use glium::uniforms::AsUniformValue;
use glium;

const SPRITE_INC: &'static str = include_str!("../shader/sprite.inc.fs");
const TEXTURE_INC: &'static str = include_str!("../shader/texture.inc.fs");
const SPRITE_VS: &'static str = include_str!("../shader/sprite.vs");
const TEXTURE_VS: &'static str = include_str!("../shader/texture.vs");

/// A fragment shader program.
pub struct Program {
    sprite_program: glium::Program,
    texture_program: glium::Program,
}

impl Program {
    /// Creates a program from a fragment shader file.
    pub fn from_file(context: &RenderContext, file: &str) -> std::io::Result<Self> {
        let mut source = String::new();
        let mut f = try!(File::open(file));
        try!(f.read_to_string(&mut source));
        Ok(Program::from_string(context, &source))
    }
    /// Creates a program from a fragment shader string.
    pub fn from_string(context: &RenderContext, source: &str) -> Self {
        let context = rendercontext::lock(context);
        create(&context.display, source)
    }
    pub fn set_uniform<T>(self: &Self, name: &str, value: T) where T: AsUniformValue {

    }
}

/// Creates a new program. Used in rendercontext creation when the full context is not yet available.
pub fn create(display: &Display, source: &str) -> Program {
    let sprite_fs = insert_template(source, SPRITE_INC);
    let texture_fs = insert_template(source, TEXTURE_INC);
    let display_handle = &display::handle(display);
    Program {
        sprite_program: create_program(display_handle, SPRITE_VS, &sprite_fs),
        texture_program: create_program(display_handle, TEXTURE_VS, &texture_fs),
    }
}

/// Private accessor to the sprite fragement shader program.
pub fn sprite(program: &Program) -> &glium::Program {
    &program.sprite_program
}

/// Private accessor to the texture fragement shader program.
pub fn texture(program: &Program) -> &glium::Program {
    &program.texture_program
}

/// Creates a shader program from given vertex- and fragment-shader sources.
fn create_program(display: &glium::Display, vertex_shader: &str, fragment_shader: &str) -> glium::Program {
    program!(display,
        140 => {
            vertex: vertex_shader,
            fragment: fragment_shader
        }
    ).unwrap()
}

/// Inserts program boilterplate code into the shader source.
fn insert_template(source: &str, template: &str) -> String {
    let mut result = String::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push_str("\n");
        if line.starts_with("#") {
            result.push_str(template);
        }
    }
    result
}
