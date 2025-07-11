//! `dots` library

mod cli;
mod config;
mod io;
mod output_path;
mod process;
mod stdx;

pub use cli::Cli;
pub use io::World;
pub use process::process;
pub use stdx::PathExt;
