//! `dots` library

mod cli;
mod config;
mod output_path;
mod process;
mod stdx;
mod world;

pub use cli::Cli;
pub use process::{WritePath, process};
pub use stdx::PathExt;
pub use world::World;
