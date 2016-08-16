use renderer::Renderer;

pub struct Sprite {
    pub anchor_x    : f32,
    pub anchor_y    : f32,
    width           : u32,
    height          : u32,
    frames          : u32,
    bucket_id       : u32,
    bucket_pos      : u32,
    u_max           : f32,
    v_max           : f32,
}

impl Sprite {
    pub fn new(width: u32, height: u32, frames: u32, bucket_pos: u32) -> Self {

        let (bucket_id, tex_size) = Renderer::bucket_info(width, height);

        Sprite {
            width       : width,
            height      : height,
            frames      : frames,
            anchor_x    : 0.5,
            anchor_y    : 0.5,
            bucket_id   : bucket_id,
            bucket_pos  : bucket_pos,
            u_max       : (width as f32 / tex_size as f32),
            v_max       : (height as f32 / tex_size as f32),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn frames(&self) -> u32 {
        self.frames
    }
    pub fn bucket_id(&self) -> u32 {
        self.bucket_id
    }
    pub fn texture_id(&self, frame_id: u32) -> u32 {
        self.bucket_pos + (frame_id % self.frames)
    }
    pub fn u_max(&self) -> f32 {
        self.u_max
    }
    pub fn v_max(&self) -> f32 {
        self.v_max
    }
}
