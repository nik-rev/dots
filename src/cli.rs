//! The CLI interface

use clap::{
    Parser,
    builder::styling::{AnsiColor, Effects},
};

/// Styles for the CLI
const STYLES: clap::builder::Styles = clap::builder::Styles::styled()
    .header(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightCyan.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightYellow.on_default().effects(Effects::BOLD));

/// Command-line interface
#[derive(Parser, Debug, Copy, Clone)]
#[command(version, styles = STYLES, long_about = None)]
#[allow(clippy::struct_excessive_bools, reason = "normal for CLIs")]
pub struct Cli {
    /// Control how much is logged
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}
