#![doc(html_logo_url = "https://raw.githubusercontent.com/sinesc/radiant-rs/master/doc/logo.png",
       html_favicon_url = "https://raw.githubusercontent.com/sinesc/radiant-rs/master/doc/favicon.png")]

/*!
Rust sprite rendering engine with a friendly API, wait-free send+sync drawing targets and custom shader support.

It is intended for use in simple 2d games or prototypes but can also be combined with additional functionality from the backend
library (currently Glium).

# Examples

Here is a short [video](https://www.youtube.com/watch?v=EcH1Sl5IuTQ&index=5&list=PLz6zhQmaeK57f67Fjw3GyxUK5gzTzpZbM) of a space
shooter/demo game using Radiant (graphics), Specs (ECS), Rodio (audio), Serde (deserializing levels, entities, ...). GitHub link in the description.

The examples folder contains multiple small examples. They can be run via `cargo run --example <example name>`, e.g.
`cargo run --example demo_blobs` to run demo_blobs.rs.

# Basic rendering

1. Create a [display](struct.Display.html) with `Display::builder()`. This represents the window/screen. **Note:** It is also
possible to use backend window/event handling instead. See further below.
2. Create a [renderer](struct.Renderer.html) with `Renderer::new()`. It is used to draw to rendertargets like the display.
3. Grab a [context](struct.Context.html) from the display using the `context()` method. It ties everything together.
4. Load [sprites](struct.Sprite.html) or [fonts](struct.Font.html) using e.g. `Font::from_file()` or `Sprite::from_file()`.
5. Create as many drawing [layers](struct.Layer.html) as you need using `Layer::new()`.
6. Draw to the layer using the `Font::write()` or `Sprite::draw()` methods.
7. Prepare a new frame and clear it using `Display::clear_frame()` (or `Display::prepare_frame()` if you don't want to clear).
8. Draw the contents of your layers to the display using `Renderer::draw_layer()`.
9. Make the frame visible via `Display::swap_frame()`.

# Multiple windows, shared context

1. Create a [context](struct.Context.html) with `Context::new()`.
2. Create as many displays as are needed using `Display::builder()` while using the builder's `context()` method to specify the
previously created context.
3. Either...
    1. Create a single headless renderer using `Renderer::headless()` and use `Renderer::render_to()` to render to a specific window or
    2. Create a renderer for each `Display`.
4. Continue with step 4. from above.

# Draw to texture/postprocess

Postprocessors are custom effects that may be as simple as a single shader program or combine multiple shaders and textures into a single
output.

The renderer has a method [`Renderer::render_to()`](struct.Renderer.html#method.render_to) that accepts a rendertarget (e.g. texture) and a closure. Anything
drawn within the closure will be rendered to the texture.

Likewise, use [`Renderer::postprocess()`](struct.Renderer.html#method.postprocess) to render using a postprocessor.

These methods can be combined/nested as shown here:

```
# use radiant_rs::*;
# let display = Display::builder().build().unwrap();
# let renderer = Renderer::new(&display).unwrap();
# let layer = Layer::new((1.0, 1.0));
# let surface = Texture::new(&display.context(), 1, 1);
# let program = Program::from_string(&display.context(), "#version 140\nout vec4 f_color;\nvoid main() { f_color = vec4(0.0, 0.0, 0.0, 0.0); }").unwrap();
# let p2 = program.clone();
# let effect1 = postprocessors::Basic::new(&display.context(), program, display.dimensions());
# let effect2 = postprocessors::Basic::new(&display.context(), p2, display.dimensions());
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

# Sprite-sheets

Currently sprite-sheets are required to be sheets of one or more either horizontally or vertically aligned sprite frames. Each frame
can have multiple components aligned orthogonally to the frames. Components could be the sprite's color image, a light or distortion
map for the shader etc.

Sprites can be created from either raw image data and a [`SpriteParameters`](support/struct.SpriteParameters.html) struct describing the
sprite layout, or directly from a file.
When loading from file, filenames are required to express the sprite format, e.g. `battery_lightmapped_128x128x15x2` would be 15 frames
of a 128x128 sprite using two components. This is a scaled version of how it could look. The color component is in the top row, a lightmap
component in the bottom row:

![Spritesheet](https://raw.githubusercontent.com/sinesc/radiant-rs/master/doc/spritesheet.png "Spritesheet")

# Custom shaders

Radiant supports the use of custom fragment shaders. These are normal glsl shaders. To simplify access to the default
sampler (which might be a sampler2DArray or sampler2D, depending on what is drawn) a wrapper is injected into the
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

# Drawing from multiple threads

Start with steps 1-5 from the *Basic rendering* list. Then...

1. Wrap fonts, sprites, and layers in `Arc`s.
2. Clone the `Arc`s for each thread that needs their contents. The context can be cloned directly.
3. Move the clones into the thread.
4. Draw onto your layers, load sprites etc. from any thread(s). Layers are non-blocking for drawing operations,
blocking for other manipulations (e.g. matrix modification).

Complete rendering with steps 7-9 from the *Basic rendering* list in the thread that created the `Renderer`; both it
and `Display` do not implement `Send`.

# Using Radiant with Glium

The [backend](backend/index.html) module provides various methods to use Radiant along side Glium. The examples glium_less and glium_more
demonstrate two possible approaches.

Approach "more": Skip creating a Radiant Display and use [`backend::create_renderer()`](backend/fn.create_renderer.html) to create a renderer from a Glium Display.
Then use [`backend::target_frame`](backend/fn.target_frame.html) to direct the renderer to target the given Glium Frame instead.

Approach "less": Use [`backend::create_display()`](backend/fn.create_display.html) to create a Radiant Display from a Glium Display. Then use
[`backend::take_frame()`](backend/fn.take_frame.html) to "borrow" a Glium Frame from Radiant. This approach let's you keep Radiant's window/event handling.

# Found and issue? Missing a feature?

Please file a bug report if you encounter any issues with this library. In particular, it has only been tested on a limited number of graphics cards
so I would expect issues regarding untested hardware.
*/

#[cfg(feature = "glium")]
#[macro_use] extern crate glium;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate lazy_static;
extern crate image;
extern crate regex;
extern crate rusttype;
extern crate unicode_normalization;
extern crate font_loader;
extern crate avec;
extern crate palette;
#[cfg(feature = "serialize-serde")]
extern crate serde;
#[cfg(feature = "serialize-serde")]
#[macro_use] extern crate serde_derive;

mod prelude;
mod backends;
mod core;

mod public;
pub use public::*;
