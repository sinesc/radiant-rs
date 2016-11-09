# radiant-rs
Very easy to use, thread-safe Rust sprite rendering engine.

This is work-in-progress. The API is incomplete and will likely still change somewhat. Have a look if you like, but don't depend on it :)

[Documentation here](https://sinesc.github.io/doc/radiant_rs/).

To compile the examples, use e.g. `cargo run --release --example blobs`. See examples folder for other available examples.

![Screenshot](https://raw.githubusercontent.com/sinesc/radiant-rs/master/res/screenshot.jpg "Screenshot")

## 10 lines to the first frame

1. Create a display with `Display::new()`. This represents the window/fullscreen.
2. Create a renderer with `Renderer::new()`. It does all the work.
3. Grab a context from the renderer using the `context()` method. It is only required in order to load resources.
4. Use it to load sprites or fonts with e.g. `Font::from_file()` or `Sprite::from_file()`.
5. Create as many drawing layers as you need using `Layer::new()`.
6. Draw stuff onto the layer using the `Font::write()` or `Sprite::draw()` methods.
7. Clear the drawing target using `Renderer::clear_target()`. (If you don't want to clear, use `Renderer::prepare_target()` instead.)
8. Draw the contents of your layers onto the target using `Renderer::draw_layer()`.
9. Make the target visible via `Renderer::swap_target()`.
10. Goto 6.

## Multi-threaded environments

1. Stick fonts, sprites, and layers or scenes into `Arc`s.
2. Clone the `Arc`s for each thread that needs their contents. The rendercontext can be cloned directly.
3. Move the clones into the thread.
4. Draw onto your layers, load sprites etc. from however many threads you like. Layers are non-blocking for drawing operations, blocking for other manipulations (e.g. matrix modification).
5. Perform steps 7-10 from the above list in the thread that created the `Renderer`; both it and `Display` do not implement `Send`
