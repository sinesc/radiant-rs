use core::*;
use maths::*;
use std::sync::Mutex;
use std::mem::swap;
use std::cmp::{min, max};

/// A simple bloom postprocessor.
/// 
/// This effect internally uses textures of decreasing dimensions to amplify an initially small blur effect
/// via linear interpolation performed by the gpu when scaling texture contents.
pub struct Bloom {
    targets         : [[Texture; 5]; 2],
    blur_program    : Mutex<Program>,
    combine_program : Program,
    /// Number of blur iterations.
    pub iterations  : u8,
    /// Blendmode to use in blur iterations.
    pub iter_blend  : BlendMode,
    /// Blendmode to use for the final drawing operation.
    pub draw_blend  : BlendMode,
    /// Color multiplicator for the final drawing operation.
    pub draw_color  : Color,
    /// Clear internal textures before processing.
    pub clear       : bool,
    /// Number of scaling steps used vertically. Limited to 5.
    pub vertical    : u8,
    /// Number of scaling steps used horizontally. Limited to 5.
    pub horizontal  : u8,
}

impl Postprocessor for Bloom {
    type T = ();

    /// Returns the target where the postprocessor expects the unprocessed input.
    fn target(self: &Self) -> &Texture {
        if self.clear {
            let horizontal = min(self.horizontal as usize, self.targets[0].len());
            let vertical = min(self.vertical as usize, self.targets[0].len());
            let spread = max(horizontal, vertical);
            for i in 0..spread as usize {
                self.targets[1][i].clear(Color::TRANSPARENT);
            }
            self.targets[0][0].clear(Color::TRANSPARENT);
        }
        &self.targets[0][0]
    }

    /// Process received data.
    fn process(self: &Self, renderer: &Renderer, _: &Self::T) {
        use std::ops::DerefMut;

        let horizontal = min(self.horizontal as usize, self.targets[0].len());
        let vertical = min(self.vertical as usize, self.targets[0].len());
        let spread = max(horizontal, vertical);

        // Copy to progressively smaller textures
        for i in 1..spread as usize {
            renderer.render_to(&self.targets[0][i], || {
                renderer.copy_from(&self.targets[0][i-1], TextureFilter::Linear);
            });
        }

        let mut blur = self.blur_program.lock().unwrap();
        let blur = blur.deref_mut();
        let mut dst = 1;
        let mut src = 0;

        for _ in 0..self.iterations {

            // Apply horizontal blur
            if horizontal > 0 {
                blur.set_uniform("horizontal", &true);
                for i in 0..spread as usize {
                    renderer.render_to(&self.targets[dst][i], || {
                        let fill = renderer.fill().blendmode(self.iter_blend).texture(&self.targets[src][i]);
                        if i < horizontal {
                            fill.program(&blur).draw();
                        } else {
                            fill.draw();
                        }
                    });
                }
                swap(&mut dst, &mut src);
            }

            // Apply vertical blur
            if vertical > 0 {
                blur.set_uniform("horizontal", &false);
                for i in 0..spread as usize {
                    renderer.render_to(&self.targets[dst][i], || {
                        let fill = renderer.fill().blendmode(self.iter_blend).texture(&self.targets[src][i]);
                        if i < vertical {
                            fill.program(&blur).draw();
                        } else {
                            fill.draw();
                        }
                    });
                }
                swap(&mut dst, &mut src);
            }
        }
    }

    /// Draw processed input. The renderer has already set the correct target.
    fn draw(self: &Self, renderer: &Renderer, _: &Self::T) {
        renderer.fill().blendmode(self.draw_blend).color(self.draw_color).program(&self.combine_program).draw();
    }
}

impl Bloom {
    /// Creates a new Bloom effect instance. 
    /// Initial texture size is computed from frame dimensions divided by `base_divider`. For each additional texture
    /// `base_divider` is multiplied by `divider_factor`.
    pub fn new(context: &RenderContext, base_divider: u32, divider_factor: u32) -> Self {
        
        let blur_program = Program::from_string(&context, include_str!("../../shader/postprocess/blur.fs")).unwrap();
        let mut combine_program = Program::from_string(&context, include_str!("../../shader/postprocess/combine.fs")).unwrap();
        let targets = Self::create_targets(context, base_divider, divider_factor);
        let max_ops = targets[0].len();

        combine_program.set_uniform("sample0", &targets[0][0]);
        combine_program.set_uniform("sample1", &targets[0][1]);
        combine_program.set_uniform("sample2", &targets[0][2]);
        combine_program.set_uniform("sample3", &targets[0][3]);
        combine_program.set_uniform("sample4", &targets[0][4]);

        Bloom {
            blur_program    : Mutex::new(blur_program),
            combine_program : combine_program,
            targets         : targets,
            iterations      : 3,
            iter_blend      : blendmodes::COPY,
            draw_blend      : blendmodes::ADD,
            draw_color      : Color::WHITE,
            clear           : true,
            vertical        : max_ops as u8,
            horizontal      : max_ops as u8,
        }
    }

    /// Rebuilds internal textures to current frame size.
    /// Initial texture size is computed from frame dimensions divided by `base_divider`. For each additional texture
    /// `base_divider` is multiplied by `divider_factor`.
    pub fn rebuild(self: &mut Self, context: &RenderContext, base_divider: u32, divider_factor: u32) {
        let targets = Self::create_targets(context, base_divider, divider_factor);
        self.combine_program.set_uniform("sample0", &targets[0][0]);
        self.combine_program.set_uniform("sample1", &targets[0][1]);
        self.combine_program.set_uniform("sample2", &targets[0][2]);
        self.combine_program.set_uniform("sample3", &targets[0][3]);
        self.combine_program.set_uniform("sample4", &targets[0][4]);
        self.targets = targets;
    }

    /// Create scaling textures.
    fn create_targets(context: &RenderContext, base_divider: u32, divider_factor: u32) -> [[Texture; 5]; 2] {

        let display = context.display();
        let Point2(width, height) = display.dimensions();
        let builder = Texture::builder(context).format(TextureFormat::F16F16F16F16);

        let f0 = base_divider;
        let f1 = f0 * divider_factor;
        let f2 = f1 * divider_factor;
        let f3 = f2 * divider_factor;
        let f4 = f3 * divider_factor;

        [ [
            builder.clone().dimensions((width / f0, height / f0)).build().unwrap(),
            builder.clone().dimensions((width / f1, height / f1)).build().unwrap(),
            builder.clone().dimensions((width / f2, height / f2)).build().unwrap(),
            builder.clone().dimensions((width / f3, height / f3)).build().unwrap(),
            builder.clone().dimensions((width / f4, height / f4)).build().unwrap(),
        ], [
            builder.clone().dimensions((width / f0, height / f0)).build().unwrap(),
            builder.clone().dimensions((width / f1, height / f1)).build().unwrap(),
            builder.clone().dimensions((width / f2, height / f2)).build().unwrap(),
            builder.clone().dimensions((width / f3, height / f3)).build().unwrap(),
            builder.clone().dimensions((width / f4, height / f4)).build().unwrap(),
        ] ]
    }
}
