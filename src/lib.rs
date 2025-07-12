//! `dots` library

mod analysis;
mod cli;
mod config;
mod output_path;
mod stdx;
mod world;

pub use analysis::WritePath;
pub use cli::Cli;
pub use stdx::PathExt;
pub use world::Link;
pub use world::World;
