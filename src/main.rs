//! `dots` is a cozy dotfiles manager

use config::Config;
use eyre::{Context as _, Result};
use simply_colored::*;

use std::io::Write as _;
mod config;
mod output_path;
mod stdx;

use log::Level;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let color = match record.level() {
                Level::Error => RED,
                Level::Warn => YELLOW,
                Level::Info => GREEN,
                Level::Debug => BLUE,
                Level::Trace => CYAN,
            };
            let level = record.level();
            let message = record.args();

            writeln!(buf, "{BLACK}[{color}{level}{BLACK}]{RESET} {message}",)
        })
        .init();

    let _ = color_eyre::install();

    Config::find()
        .context("failed to find config")?
        .process()
        .context("failed to process config")?;

    Ok(())
}
