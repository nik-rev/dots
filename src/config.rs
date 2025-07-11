//! Config for `dots`

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use eyre::Result;
use serde::{Deserialize, Serialize};
use tap::Pipe as _;

use crate::output_path::OutputPath;

/// Configuration for `dots`
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Path to the directory that contains the config file
    #[serde(skip)]
    pub root: PathBuf,
    /// A list of links
    ///
    /// File located at the link will be fetched into the appropriate location
    #[serde(rename = "link")]
    pub links: Vec<Link>,
    #[serde(rename = "dir")]
    /// List of directories to process
    pub dirs: Vec<Dir>,
}

pub const GITHUB: &str = "https://github.com/nik-rev/dots";

impl Config {
    /// Name of the config file for `dots` to search for
    pub const FILE_NAME: &str = "dots.toml";
}

/// Arguments that the marker takes
///
/// This is found on the first line of each source file of this form:
///
/// ```text
/// @dots --path '{config}/gitui/theme.ron'
/// ```
#[derive(Parser)]
pub struct Marker {
    /// Write file to this path
    #[arg(long)]
    pub path: Option<OutputPath>,
}

impl Marker {
    /// Marker to use in files to add extra info about them
    pub const MARKER: &str = "@dots ";
}

impl FromStr for Marker {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        shellwords::split(s)?.pipe(Marker::try_parse_from)?.pipe(Ok)
    }
}

/// Represents a single input and output directory to use
#[derive(Deserialize, Debug)]
pub struct Dir {
    /// Local path to a directory that will be interpreted
    pub input: PathBuf,
    /// Output directory
    pub output: OutputPath,
}

/// A link representing a file to be fetched
#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    /// URL to the link, e.g. `https://raw.githubusercontent.com/catppuccin/nushell/05987d258cb765a881ee1f2f2b65276c8b379658/themes/catppuccin_mocha.nu`
    pub url: String,
    /// Path where to write the file to in the `config` directory,
    /// e.g. `nushell/catppuccin.nu` writes to `config/nushell/catppuccin.nu` if `config` in `Config` is `"config"`
    pub path: PathBuf,
    /// Expected hash of the file. This can be supplied for security purposes
    pub sha256: Option<String>,
    /// A marker to add, like `"--path '{config}/gitui/theme.ron'"`
    /// This marker is not interpreted. Instead, the marker is written to the
    /// file as-is
    pub marker: Option<String>,
}
