
//! Various optional utility features including a Scene and mainloop helpers.

mod loops;
mod rng;

pub use utils::loops::{renderloop, mainloop};
pub use utils::rng::Rng;
pub use scene::{Scene, Operation};
