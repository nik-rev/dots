//! `dots` is a cozy dotfiles manager

use clap::Parser as _;
use dots::{Cli, World};
use eyre::{Context as _, Result, eyre};
use simply_colored::*;
use std::io::Write as _;
use tap::Pipe as _;

use log::Level;

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbosity.into())
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

    std::env::current_dir()
        .context("failed to obtain current working directory")?
        .pipe_deref(World::new)
        .and_then(World::process)
        .map_err(|errs| {
            for err in errs {
                log::error!("{err}");
            }

            eyre!("encountered errors")
        })?
        .finish();

    Ok(())
}
