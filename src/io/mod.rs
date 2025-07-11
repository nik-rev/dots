//! IO interaction.
//!
//! This module exposes a [`World`] struct, which represents all inputs to the pure program.
//! and the [`Analysis`] struct which is returned by the program, which is then processed for IO.

mod analysis;
mod world;

pub use analysis::{Analysis, WritePath};
pub use world::{File, Link, World};
