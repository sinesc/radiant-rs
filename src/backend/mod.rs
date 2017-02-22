
// temporary. ultimately, the public interface of this module needs to be backend agnostic

pub mod glium {

    use core::TextureFilter;
    use maths::Rect;
    use glium as ext;

    pub fn blit_coords(source_rect: Rect<i32>, source_height: u32, target_rect: Rect<i32>, target_height: u32) -> (ext::Rect, ext::BlitTarget) {
        (ext::Rect {
            left: (source_rect.0).0 as u32,
            bottom: (source_height as i32 - (source_rect.1).1 as i32 - (source_rect.0).1 as i32) as u32,
            width: (source_rect.1).0 as u32,
            height: (source_rect.1).1 as u32,
        },
        ext::BlitTarget {
            left: (target_rect.0).0 as u32,
            bottom: (target_height as i32 - (target_rect.1).1 as i32 - (target_rect.0).1 as i32) as u32,
            width: (target_rect.1).0 as i32,
            height: (target_rect.1).1 as i32,
        })
    }

    pub fn magnify_filter(filter: TextureFilter) -> ext::uniforms::MagnifySamplerFilter {
        if filter == TextureFilter::Linear {
            ext::uniforms::MagnifySamplerFilter::Linear
        } else {
            ext::uniforms::MagnifySamplerFilter::Nearest
        }
    }

}
