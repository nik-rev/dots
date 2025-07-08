//! Config for `dots`

use std::collections::BTreeMap;
use std::path::{self, Path};
use std::str::FromStr;
use std::{fs, path::PathBuf};

use clap::Parser;
use eyre::{Context as _, ContextCompat as _, bail};
use eyre::{Result, eyre};
use handlebars::Handlebars;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use simply_colored::*;
use tap::Pipe as _;
use walkdir::WalkDir;

use crate::stdx::{self, PathExt as _};
use crate::{output_path::OutputPath, stdx::traverse_upwards};

/// Configuration for `dots`
#[derive(Deserialize)]
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

const GITHUB: &str = "https://github.com/nik-rev/dots";

impl Config {
    /// Name of the config file for `dots` to search for
    const FILE_NAME: &str = "dots.toml";

    /// Search for a config file, starting from the root
    pub fn find() -> Result<Self> {
        let cwd = std::env::current_dir().context("failed to obtain current working directory")?;

        // Search for a root directory - which contains the config file we are searching for.
        let root = cwd
            .pipe_ref(traverse_upwards)
            .find(|dir| dir.join(Self::FILE_NAME).exists())
            .with_context(|| {
                eyre!(
                    "failed to find directory that contains a `{}`. traversed upwards from {}",
                    Self::FILE_NAME,
                    cwd.show()
                )
            })?;

        root.join(Self::FILE_NAME)
            .pipe(fs::read_to_string)
            .with_context(|| eyre!("failed to read config file {}", Self::FILE_NAME))?
            .pipe_deref(toml::de::from_str::<Self>)
            .context("failed to parse config file")?
            .pipe(|mut conf| {
                conf.root = root;
                conf
            })
            .pipe(Ok)
    }

    /// Process the config
    ///
    /// - Download all links to the given locations
    /// - Map all input paths to output paths
    pub fn process(self) -> Result<()> {
        self.links
            .into_iter()
            .map(|link| link.download(&self.root))
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .pipe(|(_, errors)| {
                for error in &errors {
                    log::error!("{error}");
                }
                if !errors.is_empty() {
                    bail!("encountered errors when downloading links");
                }

                Ok(())
            })?;

        self.dirs
            .into_iter()
            .map(|dir| dir.process(&self.root))
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .pipe(|(_, errors)| {
                for err in &errors {
                    log::error!("{err}");
                }
                if !errors.is_empty() {
                    bail!("encountered errors when trying to proccess directories")
                }

                Ok(())
            })?;

        Ok(())
    }
}

/// Arguments that the marker takes
///
/// This is found on the first line of each source file of this form:
///
/// ```text
/// @dots --path '{config}/gitui/theme.ron'
/// ```
#[derive(Parser)]
struct Marker {
    /// Write file to this path
    #[arg(long)]
    path: Option<OutputPath>,
}

impl Marker {
    /// Marker to use in files to add extra info about them
    const MARKER: &str = "@dots ";
}

impl FromStr for Marker {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        shellwords::split(s)?.pipe(Marker::try_parse_from)?.pipe(Ok)
    }
}

/// Represents a single input and output directory to use
#[derive(Deserialize)]
pub struct Dir {
    /// Local path to a directory that will be interpreted
    pub input: PathBuf,
    /// Output directory
    pub output: OutputPath,
}

impl Dir {
    /// Process the directory
    pub fn process(self, root: &Path) -> Result<()> {
        let Self { input, output } = self;

        WalkDir::new(input)
            .into_iter()
            .flatten()
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .try_for_each(|file| {
                let old_location = path::absolute(file.path())?;

                let file_contents = fs::read_to_string(&old_location)
                    .with_context(|| eyre!("failed to read path {}", old_location.show()))?;

                let relative_location = old_location.strip_prefix(root).with_context(|| {
                    eyre!(
                        "failed to strip prefix {} from {}",
                        root.show(),
                        old_location.show()
                    )
                })?;

                let (file_contents, new_location) = if let Some(first_line) =
                    file_contents.lines().next()
                    && let Some(marker_start_pos) = first_line.find(Marker::MARKER)
                    && let Some(marker_args) =
                        first_line.get(marker_start_pos + Marker::MARKER.len()..)
                    && let Ok(args) = marker_args.parse::<Marker>()
                    && let Some(path) = args.path
                {
                    (
                        // remove the first line which contains the `@dotfilers`
                        file_contents.lines().skip(1).collect_vec().join(","),
                        path,
                    )
                } else {
                    (
                        file_contents,
                        output
                            .as_ref()
                            .join(relative_location)
                            .pipe(OutputPath::new),
                    )
                };

                let mut handlebars = Handlebars::new();
                handlebars
                    .register_template_string("t1", file_contents)
                    .with_context(|| eyre!("failed to parse template for {new_location}"))?;

                let contents = handlebars
                    .render("t1", &BTreeMap::<u8, u8>::new())
                    .with_context(|| eyre!("failed to render template for {new_location}"))?;

                stdx::write_file(new_location.as_ref(), &contents)
                    .context("failed to write output file")?;

                Ok::<_, eyre::Error>(())
            })
    }
}

/// A link representing a file to be fetched
#[derive(Serialize, Deserialize)]
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

impl Link {
    /// Process the link
    pub fn download(self, root: &Path) -> Result<()> {
        let url = &self.url;

        let link_contents = ureq::get(&self.url).call()?.body_mut().read_to_string()?;
        let actual_sha256 = sha256::digest(&link_contents);

        if let Some(expected_sha256) = self.sha256
            && actual_sha256 != *expected_sha256
        {
            let mismatch = format!("link       {BLUE}{url}{RESET}");
            let actual = format!("actual     {CYAN}{actual_sha256}{RESET}");
            let expected = format!("expected   {CYAN}{expected_sha256}{RESET}");
            bail!("hash mismatch\n  {mismatch}\n  {actual}\n  {expected}");
        }

        // download the link's contents to *this* path
        let path = root.join(self.path);

        // add the marker if necessary
        let marker = self.marker.as_ref().map_or(String::new(), |marker_args| {
            format!("{}{marker_args}", Marker::MARKER)
                .pipe(|marker| commented::comment(marker, &path))
                + "\n"
        });

        let file_contents = format!("{marker}{link_contents}");

        let marker = file_contents
            .lines()
            .next()
            .filter(|line| line.contains(Marker::MARKER))
            .map(|line| format!("{line}\n"))
            .unwrap_or_default();

        let contents = if let Some(first_line) = file_contents.lines().next()
            && first_line.contains(Marker::MARKER)
        {
            file_contents.lines().skip(1).collect::<Vec<_>>().join("\n")
        } else {
            file_contents.to_string()
        };

        let generated_notice = [
            format!("@generated by `{}` <{GITHUB}>", Marker::MARKER),
            "Do not edit by hand.".to_string(),
            String::new(),
            format!("downloaded from: {url}"),
        ]
        .into_iter()
        .fold(String::new(), |previous_lines, line| {
            format!("{previous_lines}{}\n", commented::comment(line, &path))
        });

        let contents = format!("{marker}{generated_notice}{contents}");

        stdx::write_file(&path, &contents)?;

        log::info!(
            "{CYAN}downloaded{RESET}\n  {BLUE}{url}{RESET} {BLACK}\n  ->{RESET}  {}",
            path.show()
        );

        Ok::<_, eyre::Error>(())
    }
}
