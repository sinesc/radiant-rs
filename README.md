# radiant-rs
Rust sprite rendering engine. Main goals: threadsafty, API-simplicity.

This is work-in-progress. API is still incomplete and will probably change heavily. Don't bother with it yet :)

```rust
use radiant_rs::{Renderer, Color, Vec3};

let mut renderer = Renderer::new(glium_display/* a glium Display */, 1000);
let sprite = renderer.texture(r"/path/to/some/sprite/sheet_256x256x40.jpg");
let mut layer = renderer.layer();
let mut frame = 0;

loop {
  // add some sprites to the layer
  layer.sprite(&sprite, frame, 500, 500, Color::white(), 0.0, 1.0, 1.0);
  frame += 1;
  
  // do some transforms with the layer
  layer.matrix
       .translate(&Vec3(320.0, 220.0, 0.0))
       .rotate_z(0.01)
       .translate(&Vec3(-320.0, -220.0, 0.0));  

  // clear render target and draw the layer
  renderer.prepare_target();
  renderer.clear_target(&Color::black());
  layer.draw().reset();
  renderer.swap_target();    
}
```
