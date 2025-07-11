//! `dots` is a cozy dotfiles manager

use clap::Parser as _;
use dots::process;
use dots::{Cli, PathExt as _, World};
use eyre::{Context as _, ContextCompat as _, Result, eyre};
use simply_colored::*;

use std::{
    fs,
    io::{self, Write as _},
};

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

    let writes = World::new().and_then(process).map_err(|errs| {
        for err in errs {
            log::error!("{err}");
        }

        eyre!("encountered errors")
    })?;

    for dots::WritePath { path, contents } in writes {
        let contents = contents.to_string();

        match fs::remove_file(&path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
            Ok(()) => Ok(()),
        }
        .with_context(|| eyre!("failed to remove file: {}", path.show()))?;

        log::warn!("{RED}removed{RESET} {}", path.show());

        let dir = path
            .parent()
            .with_context(|| eyre!("failed to obtain parent of {}", path.show()))?;

        // 2. Create parent directory which will contain the file downloaded from the link
        fs::create_dir_all(dir)
            .with_context(|| eyre!("failed to create directory for {}", dir.show()))?;

        fs::write(&path, contents).with_context(|| eyre!("failed to write to {}", path.show()))?;

        log::info!("wrote to {}", path.show());
    }

    Ok(())
}
