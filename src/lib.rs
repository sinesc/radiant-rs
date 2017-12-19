#![doc(html_logo_url = "https://sinesc.github.io/images/radiant-logo.png",
       html_favicon_url = "https://sinesc.github.io/images/radiant-favicon.png")]

/*!
Rust sprite rendering engine with a friendly API, wait-free send+sync drawing targets and custom shader support.

To view the reference, scroll down or collapse this block using the [-] to the left.

# Examples

The examples folder contains multiple small examples. They can be run via `cargo run --example <example name>`, e.g.

`cargo run --release --example demo_blobs` to run demo_blobs.rs

# Basic rendering

1. Create a [display](struct.Display.html) with `Display::builder()`. This represents the window/screen.
2. Create a [renderer](struct.Renderer.html) with `Renderer::new()`. It is used to draw to rendertargets like the display.
3. Grab a context from the renderer using the `context()` method. It is required for resource loading.
4. Load [sprites](struct.Sprite.html) or [fonts](struct.Font.html) using e.g. `Font::from_file()` or `Sprite::from_file()`.
5. Create as many drawing [layers](struct.Layer.html) as you need using `Layer::new()`.
6. Draw to the layer using the `Font::write()` or `Sprite::draw()` methods.
7. Prepare a new frame and clear it using `Display::clear_frame()`. (If you don't want to clear, use `Display::prepare_frame()` instead.)
8. Draw the contents of your layers to the display using `Renderer::draw_layer()`.
9. Make the frame visible via `Display::swap_frame()`.
10. Consider clearing the layer and goto 6. Or maybe simply change some layer properties and redraw it starting a step 7.

# Integrating with existing glium projects (or any supported backend)

Radiant can be integrated with supported backends using the APIs provided in the [backend](backend/index.html) module. The 10_glium example shows how
to do this with Glium.

These APIs are currently experimental and likely subject to change.

# Draw to texture/postprocess

Postprocessors are custom effects that may be as simple as a single shader program or combine multiple shaders and textures into a single
output.

The renderer has a method [`Renderer::render_to()`](struct.Renderer.html#method.render_to) that accepts a texture and a closure. Anything
drawn within the closure will be rendered to the texture.

Likewise, use [`Renderer::postprocess()`](struct.Renderer.html#method.postprocess) to render using a postprocessor.

These methods can be combined/nested as shown here:

```
# use radiant_rs::*;
# let display = Display::builder().build();
# let renderer = Renderer::new(&display).unwrap();
# let layer = Layer::new((1.0, 1.0));
# let surface = Texture::new(&renderer.context(), 1, 1);
# let program = Program::from_string(&renderer.context(), "#version 140\nout vec4 f_color;\nvoid main() { f_color = vec4(0.0, 0.0, 0.0, 0.0); }").unwrap();
# let p2 = program.clone();
# let effect1 = postprocessors::Basic::new(&renderer.context(), program);
# let effect2 = postprocessors::Basic::new(&renderer.context(), p2);
# let effect1_arguments = blendmodes::ALPHA;
# let effect2_arguments = blendmodes::ALPHA;
renderer.render_to(&surface, || {
    renderer.postprocess(&effect1, &effect1_arguments, || {
        renderer.postprocess(&effect2, &effect2_arguments, || {
            //...
            renderer.draw_layer(&layer, 1);
        });
        //... maybe draw here with only effect 1? ...
    });
    //... or here without any postprocessor? ...
});
```

# Drawing from multiple threads

1. Wrap fonts, sprites, and layers in `Arc`s.
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

Radiant supports the use of custom fragment shaders. These are normal glsl shaders. To simplify access to the default
sampler (which might be a sampler2DArray or sampler2D, depending on what is drawn) a tiny wrapper is injected into the
source. The wrapper provides `sheet*()` functions similar to glsl's `texture*()` functions.
This only applies to the default sampler. It is possible to add custom uniforms, including samplers, to your shader
that would be sampled using the `texture*()` functions.

Available default inputs:

- `uniform mat4 u_view` The view matrix if applicable, otherwise the identity.
- `uniform mat4 u_model` The model matrix if applicable, otherwise the identity.
- `in vec2 v_tex_coords` Texture coordinates.
- `in vec4 v_color` Color multiplier. For layers this is sprite color * layer color.

To access the default sampler, the following wrappers are provided:

- `vec2 sheetSize()` Retrieves the dimensions of the texture.
- `vec4 sheet(in vec2 texture_coords)` Retrieves texels from the texture.
- `vec4 sheetComponent(in vec2 texture_coords, in uint component)` Samples a specific sprite
component instead of the default one set by `Renderer::draw_layer()`.
- `vec4 sheetOffset(in vec2 texture_coords, in ivec2 offset)` Like textureOffset().

Example: (This is the default shader used by radiant.)

```text
#version 140

in vec2 v_tex_coords;
in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = sheet(v_tex_coords) * v_color;
}
```
*/

#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate lazy_static;
extern crate image;
extern crate regex;
extern crate num_traits;
extern crate rusttype;
extern crate unicode_normalization;
extern crate font_loader;
extern crate avec;
extern crate palette;

mod prelude;
mod backends;
mod core;
mod maths;
mod misc;

mod public;
pub use public::*;
