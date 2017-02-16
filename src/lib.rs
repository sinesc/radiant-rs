#![doc(html_logo_url = "https://sinesc.github.io/images/radiant-logo.png",
       html_favicon_url = "https://sinesc.github.io/images/radiant-favicon.png")]

/*!
Thread-safe Rust sprite rendering engine with an easy to use API and custom shader support.

To view the reference, scroll down or collapse this block using the [-] to the left.

**This is work-in-progress. Parts of the API are incomplete and will likely still change somewhat.**

# Examples

The examples folder contains multiple small examples. They can be run via `cargo run --example <example name>`, e.g.

`cargo run --example blobs` to run blobs.rs

# Basic rendering

1. Create a [display](struct.Display.html) with `Display::new()`. This represents the window/screen.
2. Create a [renderer](struct.Renderer.html) with `Renderer::new()`. It is used to draw to rendertargets like the display.
3. Grab a context from the renderer using the `context()` method. It is required for resource loading.
4. Load [sprites](struct.Sprite.html) or [fonts](struct.Font.html) using e.g. `Font::from_file()` or `Sprite::from_file()`.
5. Create as many drawing [layers](struct.Layer.html) as you need using `Layer::new()`.
6. Draw to the layer using the `Font::write()` or `Sprite::draw()` methods.
7. Prepare a new frame and clear it using `Display::clear_frame()`. (If you don't want to clear, use `Display::prepare_frame()` instead.)
8. Draw the contents of your layers to the display using `Renderer::draw_layer()`.
9. Make the frame visible via `Display::swap_frame()`.
10. Consider clearing the layer and goto 6. Or maybe simply change some layer properties and redraw it starting a step 7.

# Drawing from multiple threads

1. Wrap fonts, sprites, and layers or scenes in `Arc`s.
2. Clone the `Arc`s for each thread that needs their contents. The rendercontext can be cloned directly.
3. Move the clones into the thread.
4. Draw onto your layers, load sprites etc. from any thread(s). Layers are non-blocking for drawing operations,
blocking for other manipulations (e.g. matrix modification).
5. Perform steps 7-10 from the above list in the thread that created the `Renderer`; both it and `Display` do not implement `Send`.

# Sprite-sheets

Currently sprite-sheets are required to be sheets of one or more either horizontally or vertically aligned sprite frames. Each frame
can have multiple components aligned orthogonally to the frames. Components could be the sprite's color image, a light or distortion
map for the shader etc.

Filenames are required to express the sprite format, e.g. `battery_lightmapped_128x128x15x2` would be 15 frames of a 128x128 sprite using
two components. This is a scaled version of how it could look. The Color component is in the top row, a lightmap component in the bottom
row:

![Spritesheet](https://sinesc.github.io/images/spritesheet.png "Spritesheet")

# Custom shaders

Radiant supports the use of custom fragment shaders. When you create a program, a tiny wrapper is injected into the source to
make it compatible with the different internal shader programs used by the library. Instead of `texture()` you would then use `sheet()` to
retrieve data from the current texture, etc. (This is required as the texture might come from a sampler or sampler array.)
It is possible to add custom uniforms, including textures, to your shader. The postprocessing example uses a shader that takes
5 samplers and combines them.

Available default inputs:

- `uniform mat4 u_view` The view matrix if applicable, otherwise the identity.
- `uniform mat4 u_model` The model matrix if applicable, otherwise the identity.
- `in vec2 v_tex_coords` Texture coordinates.
- `in vec4 v_color` Color multiplier. For layers this is sprite color * layer color.

Wrappers:

- `vec2 sheetSize()` Retrieves the dimensions of the texture. Replaces `textureSize()`.
- `vec4 sheet(in vec2 texture_coords)` Retrieves texels from the texture. Replaces `texture()`
- !todo add other texture*.

Example: (This is the default shader used by radiant.)

```
#version 140

in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = sheet(v_tex_coords) * v_color;
}
```

*/

#[macro_use] extern crate glium;
extern crate image;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate num;
extern crate rusttype;
extern crate unicode_normalization;
extern crate font_loader;
extern crate avec;
#[macro_use] extern crate enum_primitive;
extern crate palette;

mod prelude;
mod core;
mod maths;
mod misc;

mod public;
pub use public::*;
