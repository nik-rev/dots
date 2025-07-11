//! `dots` library

mod analysis;
mod cli;
mod config;
mod output_path;
mod process;
mod stdx;
mod world;

pub use cli::Cli;
pub use process::process;
pub use stdx::PathExt;
pub use world::World;
