
//! Various optional utility features.
//!
//! These features are not required in order to use radiant_rs. Some of them are used internally
//! by the library and published here because they might be useful to the user. Others are intented
//! to help early prototyping.

mod loops;
mod rng;

pub use avec::AVec;
pub use utils::loops::{renderloop, mainloop};
pub use utils::rng::Rng;
