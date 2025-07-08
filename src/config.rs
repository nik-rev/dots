//! Config for `dots`

use std::collections::BTreeMap;
use std::path::Path;
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

use crate::stdx;
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
                    cwd.display()
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
    pub fn process(self) -> Result<()> {
        self.links
            .into_iter()
            .map(|link| link.process(&self.root))
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .pipe(|(_, errors)| {
                for error in &errors {
                    log::error!("{error}");
                }
                if errors.is_empty() {
                    Err(eyre!("encountered errors when downloading links"))
                } else {
                    Ok(())
                }
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
    pub path: PathBuf,
    /// Output directory
    pub output: OutputPath,
}

impl Dir {
    /// Process the directory
    pub fn process(self, root: &Path) -> Result<()> {
        let Self { path, output } = self;

        WalkDir::new(path)
            .into_iter()
            .flatten()
            .filter(|dir_entry| dir_entry.file_type().is_file())
            .try_for_each(|file| {
                let old_location = file.path();

                let file_contents = fs::read_to_string(old_location)
                    .with_context(|| eyre!("failed to read path {}", old_location.display()))?;

                let relative_location = old_location.strip_prefix(root).with_context(|| {
                    eyre!(
                        "failed to strip prefix {} from {}",
                        root.display(),
                        old_location.display()
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

                let old_relative_to_cwd_canon = fs::canonicalize(old_location)
                    .with_context(|| eyre!("failed to canonicalize {}", old_location.display()))?;
                let old_relative_to_cwd = old_relative_to_cwd_canon
                    .strip_prefix(root)
                    .with_context(|| {
                        eyre!(
                            "failed to strip prefix {} from {}",
                            root.display(),
                            old_relative_to_cwd_canon.display()
                        )
                    })?
                    .display();

                // 1. Remove the old file
                stdx::remove_file(new_location.as_ref())
                    .inspect(|()| log::warn!("{RED}removed{RESET} {old_relative_to_cwd}"))?;

                let dir = new_location
                    .as_ref()
                    .parent()
                    .with_context(|| eyre!("failed to obtain parent of {new_location}"))?;

                // 2. Parent directory of existing file might not exit
                //
                //    We don't want to symlink directories themselves,
                //    because they might contain data we don't want in
                //    our dotfiles.
                fs::create_dir_all(dir)
                    .with_context(|| eyre!("failed to create {}", dir.display()))?;

                let mut handlebars = Handlebars::new();
                handlebars
                    .register_template_string("t1", file_contents)
                    .with_context(|| eyre!("failed to parse template for {new_location}"))?;

                let contents = handlebars
                    .render("t1", &BTreeMap::<u8, u8>::new())
                    .with_context(|| eyre!("failed to render template for {new_location}"))?;

                fs::write(new_location.as_ref(), contents)
                    .with_context(|| eyre!("failed to write to {new_location}"))?;

                log::info!(
                    "{CYAN}symlinked{RESET} \n  {old_relative_to_cwd} \
                    {BLACK}\n  ->  {RESET}{new_location}",
                );

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
    pub fn process(self, root: &Path) -> Result<()> {
        let Self {
            url,
            path,
            sha256,
            marker,
        } = self;

        let contents = ureq::get(&url).call()?.body_mut().read_to_string()?;
        let actual_sha256 = sha256::digest(&contents);

        if let Some(sha256) = sha256
            && actual_sha256 != *sha256
        {
            let mismatch = format!("link       {BLUE}{url}{RESET}");
            let actual = format!("actual     {CYAN}{actual_sha256}{RESET}");
            let expected = format!("expected   {CYAN}{sha256}{RESET}");
            bail!("hash mismatch\n  {mismatch}\n  {actual}\n  {expected}");
        }
        let path = root.join(path);

        // add the marker if necessary
        let marker = marker.as_ref().map_or(String::new(), |v| {
            format!("{}{v}", Marker::MARKER).pipe(|it| commented::comment(it, &path)) + "\n"
        });
        let contents = format!("{marker}{contents}");
        stdx::remove_file(&path)
            .with_context(|| eyre!("failed to remove file: {}", path.display()))?;

        let dir = path
            .parent()
            .with_context(|| eyre!("failed to obtain parent of {}", path.display()))?;
        fs::create_dir_all(dir)
            .with_context(|| eyre!("failed to create directory for {}", dir.display()))?;

        fs::write(&path, {
            let marker = contents
                .lines()
                .next()
                .filter(|line| line.contains(Marker::MARKER))
                .map(|line| format!("{line}\n"))
                .unwrap_or_default();
            let contents = if let Some(first_line) = contents.lines().next()
                && first_line.contains(Marker::MARKER)
            {
                contents.lines().skip(1).collect::<Vec<_>>().join("\n")
            } else {
                contents.to_string()
            };

            let header = [
                format!("@generated by `{}`", Marker::MARKER),
                "Do not edit by hand.".to_string(),
                String::new(),
                format!("downloaded from: {url}"),
            ];
            header
                .into_iter()
                .fold(String::new(), |contents, line| {
                    format!("{contents}{}\n", commented::comment(line, &path))
                })
                .pipe(|comment| format!("{marker}{comment}{contents}"))
        })
        .with_context(|| eyre!("failed to write to {}", path.display()))?;

        log::info!(
            "{CYAN}downloaded{RESET}\n  {BLUE}{url}{RESET} {BLACK}\n  ->{RESET}  {}",
            path.display()
        );

        Ok::<_, eyre::Error>(())
    }
}
