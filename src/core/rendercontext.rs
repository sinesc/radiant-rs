use core::{self, Display, font, SpriteData, Vertex};
use prelude::*;
use std::default::Default;
use backends::backend;

/// Number of texture buckets. Also requires change to renderer.rs at "let uniforms = uniform! { ... }"
pub const NUM_BUCKETS: usize = 6;

/// Initial sprite capacity. Automatically increases.
pub const INITIAL_CAPACITY: usize = 512;

/// Texture generation (increases each cleanup)
static GENERATION: AtomicUsize = ATOMIC_USIZE_INIT;

/// A thread-safe render-context.
///
/// Required to load fonts or sprites and aquired from [`Renderer::context()`](struct.Renderer.html#method.context).
#[derive(Clone)]
pub struct RenderContext (Arc<Mutex<RenderContextData>>);
unsafe impl Send for RenderContext { }
unsafe impl Sync for RenderContext { }

impl RenderContext {
    /// Retrieves the display associated with this rendercontext.
    pub fn display(self: &Self) -> Display {
        lock(self).display.clone()
    }
    /// Prunes no longer used textures. Requires all layers to be cleared before
    /// adding new sprites or rendering the layer.
    pub fn prune(self: &Self) {
        lock(self).prune();
    }
}

pub fn new(data: RenderContextData) -> RenderContext {
    RenderContext(Arc::new(Mutex::new(data)))
}

pub fn lock<'a>(context: &'a RenderContext) -> MutexGuard<'a, RenderContextData> {
    context.0.lock().unwrap()
}

/// Individual Texture.
#[derive(Clone)]
pub struct RawFrame {
    pub data    : Vec<u8>,
    pub width   : u32,
    pub height  : u32,
}

/// A weak reference back to a sprite.
struct SpriteBackRef (Weak<SpriteData>);

impl SpriteBackRef {
    /// Creates a new weak reference to SpriteData.
    fn new(data: Weak<SpriteData>) -> SpriteBackRef {
        SpriteBackRef(data)
    }
    /// Returns a strong reference to the SpriteData.
    fn upgrade(self: &Self) -> Option<Arc<SpriteData>> {
        self.0.upgrade()
    }
    /// Returns the texture id-range used by the referenced sprite or None, if it dropped.
    fn range(self: &Self) -> Option<(usize, usize)> {
        if let Some(data) = self.upgrade() {
            Some((data.texture_id.load(Ordering::Relaxed), data.num_frames as usize * data.components as usize))
        } else {
            None
        }
    }
}

/// Texture data for a single texture array.
pub struct RawFrameArray {
    pub dirty   : bool,
    pub data    : Rc<backend::Texture2dArray>,
    pub raw     : Vec<RawFrame>,
    sprites     : Vec<SpriteBackRef>,
}

impl RawFrameArray {
    fn new(display: &Display) -> Self {
        RawFrameArray {
            dirty   : false,
            data    : Rc::new(backend::Texture2dArray::new(display.handle(), &Vec::new())), // !todo why rc?
            raw     : Vec::new(),
            sprites : Vec::new(),
        }
    }
    /// Store given frames to texture arrays.
    pub fn store_frames<'a>(self: &mut Self, raw_frames: Vec<RawFrame>) -> u32 {
        let texture_id = self.raw.len() as u32;
        for frame in raw_frames {
            self.raw.push(frame);
        }
        self.dirty = true;
        texture_id
    }
    /// Stores a weak sprite reference in the context so that the sprite's texture_id can be updated after a cleanup.
    pub fn store_sprite(self: &mut Self, sprite_data: Weak<SpriteData>) {
        self.sprites.push(SpriteBackRef::new(sprite_data));
    }
    /// Updates texture array in video memory.
    fn update(self: &mut Self, display: &Display) {
        if self.dirty {
            self.dirty = false;
            self.data = Rc::new(backend::Texture2dArray::new(display.handle(), &self.raw));
        }
    }
    /// Returns a list of tuples containing current sprite texture_id and required negative offset.
    fn create_prune_map(self: &Self) -> Option<Vec<(usize, usize)>> {
        let mut mapping = self.sprites.iter().filter_map(|sprite| sprite.range()).collect::<Vec<(usize, usize)>>();
        mapping.sort_by_key(|a| a.0);
        let mut num_items = 0;
        for i in 0..mapping.len() {
            let items = mapping[i].1;
            mapping[i].1 = mapping[i].0 - num_items;
            num_items += items;
        }
        if mapping.len() > 0 { Some(mapping) } else { None }
    }
    // Shrinks raw data array using given prune-map. Returns hashmap mapping old texture index -> new texture index.
    fn prune_raw_textures(self: &mut Self, mapping: &Vec<(usize, usize)>) -> HashMap<usize, usize> {
        let new_size = self.raw.len() - mapping.last().unwrap().1;
        let mut destination_map = HashMap::new();
        for m in 0..mapping.len() {
            destination_map.insert(mapping[m].0, mapping[m].0 - mapping[m].1);
            let end = if m + 1 < mapping.len() { mapping[m+1].0 } else { new_size -1 };
            for i in (mapping[m].0)..end {
                let destination_index = i - mapping[m].1;
                self.raw.swap(i, destination_index);
            }
        }
        self.raw.truncate(new_size);
        destination_map
    }
    // Runs func on all sprites still referenced, removes unreferenced sprites from list.
    fn prune_sprites<T>(self: &mut Self, mut func: T) where T: FnMut(&Arc<SpriteData>) {
        let mut removed = Vec::new();
        for (i, sprite) in self.sprites.iter().enumerate() {
            if let Some(sprite) = sprite.upgrade() {
                func(&sprite);
            } else {
                removed.push(i);
            }
        }
        for index in removed.iter().rev() {
            self.sprites.swap_remove(*index);
        }
    }
    /// Prunes no longer used textures from the array and update sprite texture ids and generations.
    fn prune(self: &mut Self, display: &Display, generation: usize) {
        if let Some(mapping) = self.create_prune_map() {
            // Remove unused textures from raw data.
            let destination_map = self.prune_raw_textures(&mapping);
            self.dirty = true;
            self.update(display);
            // Update sprite texture ids and generation.
            self.prune_sprites(|sprite| {
                let texture_id = sprite.texture_id.load(Ordering::Relaxed);
                if let Some(new_texture_id) = destination_map.get(&texture_id) {
                    sprite.texture_id.store(*new_texture_id, Ordering::Relaxed);
                }
                sprite.generation.store(generation, Ordering::Relaxed);
            });
        } else {
            // Texure ids have not changed, simply update generation.
            self.prune_sprites(|sprite| {
                sprite.generation.store(generation, Ordering::Relaxed);
            })
        }
    }
}

/// Internal data of a RenderContext
pub struct RenderContextData {
    pub backend_context     : backend::Context,
    pub tex_arrays          : Vec<RawFrameArray>,
    pub display             : Display,
    pub font_cache          : font::FontCache,
    pub font_texture        : Rc<backend::Texture2d>,
    pub single_rect         : [core::Vertex; 4],
    generation              : usize,
}

impl RenderContextData {

    /// Create a new instance
    pub fn new(display: &Display, initial_capacity: usize) -> core::Result<Self> {

        let mut tex_arrays = Vec::new();

        for _ in 0..NUM_BUCKETS {
            tex_arrays.push(RawFrameArray::new(display));
        }

        Ok(RenderContextData {
            backend_context     : backend::Context::new(&display.handle(), initial_capacity),
            tex_arrays          : tex_arrays,
            display             : display.clone(),
            font_cache          : font::FontCache::new(512, 512, 0.01, 0.01),
            font_texture        : Rc::new(backend::Texture2d::font_cache(&display.handle(), 512, 512)),
            single_rect         : Self::create_single_rect(),
            generation          : Self::create_generation(),
        })
    }

    /// Returns the context's generation.
    pub fn generation(self: &Self) -> usize {
        self.generation
    }

    /// Update font-texture from cache
    pub fn update_font_cache(self: &Self) {
        self.font_cache.update(&self.font_texture);
    }

    /// Update texture arrays from registered textures
    pub fn update_tex_array(self: &mut Self) {
        for ref mut array in self.tex_arrays.iter_mut() {
            array.update(&self.display);
        }
    }

    /// Store given frames to texture arrays
    pub fn store_frames(self: &mut Self, bucket_id: u32, raw_frames: Vec<RawFrame>) -> u32 {
        self.tex_arrays[bucket_id as usize].store_frames(raw_frames)
    }

    /// Stores a weak sprite reference in the context so that the sprite's texture_id can be updated after a cleanup.
    pub fn store_sprite(self: &mut Self, bucket_id: u32, sprite_data: Weak<SpriteData>) {
        self.tex_arrays[bucket_id as usize].store_sprite(sprite_data);
    }

    /// Prunes no longer used textures for all texture arrays.
    fn prune(self: &mut Self) {
        self.generation = Self::create_generation();
        for array in self.tex_arrays.iter_mut() {
            array.prune(&self.display, self.generation);
        }
    }

    /// creates a single rectangle vertex buffer
    fn create_single_rect() -> [core::Vertex; 4] {
        [
            Vertex { position: [ 0.0,  0.0 ], texture_uv: [ 0.0, 1.0 ], ..Vertex::default() },
            Vertex { position: [ 1.0,  0.0 ], texture_uv: [ 1.0, 1.0 ], ..Vertex::default() },
            Vertex { position: [ 0.0,  1.0 ], texture_uv: [ 0.0, 0.0 ], ..Vertex::default() },
            Vertex { position: [ 1.0,  1.0 ], texture_uv: [ 1.0, 0.0 ], ..Vertex::default() },
        ]
    }

    // Creates a new generation and returns it
    fn create_generation() -> usize {
        // needs to start at 1 as 0 has special meaning
        GENERATION.fetch_add(1, Ordering::Relaxed) + 1
    }
}
