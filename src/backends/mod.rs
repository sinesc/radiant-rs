#[cfg(feature = "backend-wgpu")]
mod wgpu;

#[cfg(feature = "backend-wgpu")]
pub mod backend {
    pub use super::wgpu::*;
}

#[cfg(feature = "backend-null")]
mod null;

#[cfg(feature = "backend-null")]
pub mod backend {
    pub use super::null::*;
}
