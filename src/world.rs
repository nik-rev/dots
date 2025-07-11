//! Contains [`World`]

use std::{
    fs,
    path::{self, PathBuf},
};

use eyre::{Context as _, ContextCompat as _, Error, Result, eyre};
use itertools::Itertools as _;
use tap::Pipe as _;
use walkdir::WalkDir;

use crate::{
    Config,
    output_path::OutputPath,
    stdx::{self, PathExt as _},
};

/// Input to the app
#[derive(Debug)]
pub struct World {
    /// Path which contains the config file
    pub root: PathBuf,
    /// Contents of the config file
    pub links: Vec<Link>,
    /// Files to create
    pub files: Vec<File>,
}

/// Represents a URL
#[derive(Debug)]
pub struct Link {
    /// Url to the contents of the link
    pub url: String,
    /// Content at the URL
    pub contents: String,
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

/// A single file to be mapped from the input (`old_location`) to the output (`new_location`)
#[derive(Debug)]
pub struct File {
    /// Old location of the file
    pub old_location: PathBuf,
    /// Contents of the file
    pub contents: String,
    /// Output path
    pub output: OutputPath,
    /// Input directory
    pub input: PathBuf,
}

impl World {
    /// Create the `World`
    pub fn new() -> Result<Self, Vec<Error>> {
        let cwd = std::env::current_dir().map_err(single_err)?;
        // Directory which contains the config file
        let root = cwd
            .pipe_ref(stdx::traverse_upwards)
            .find(|dir| dir.join(Config::FILE_NAME).exists())
            .with_context(|| {
                eyre!(
                    "failed to find directory that contains a `{}`. traversed upwards from {}",
                    Config::FILE_NAME,
                    cwd.show()
                )
            })
            .map_err(single_err)?;

        let config = root
            .join(Config::FILE_NAME)
            .pipe(fs::read_to_string)
            .with_context(|| eyre!("failed to read config file {}", Config::FILE_NAME))
            .map_err(single_err)?
            .pipe_deref(toml::de::from_str::<Config>)
            .context("failed to parse config file")
            .map_err(single_err)?
            .pipe(|mut conf| {
                conf.root = root;
                conf
            });

        let mut errors = vec![];

        let links = config
            .links
            .into_iter()
            .map(
                |crate::config::Link {
                     url,
                     path,
                     sha256,
                     marker,
                 }| {
                    Ok::<_, Error>(Link {
                        contents: ureq::get(&url).call()?.body_mut().read_to_string()?,
                        path,
                        sha256,
                        marker,
                        url,
                    })
                },
            )
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .pipe(|(oks, errs)| {
                errors.extend(errs);
                oks
            });

        let files = config
            .dirs
            .into_iter()
            .flat_map(|crate::config::Dir { input, output }| {
                WalkDir::new(config.root.join(&input))
                    .into_iter()
                    .flatten()
                    .filter(|dir_entry| dir_entry.file_type().is_file())
                    .map(move |file| {
                        // location of the `input` file
                        let old_location = path::absolute(file.path())?;

                        let contents = fs::read_to_string(&old_location).with_context(|| {
                            eyre!("failed to read path {}", old_location.show())
                        })?;

                        Ok::<_, Error>(File {
                            old_location,
                            contents,
                            output: output.clone(),
                            input: input.clone(),
                        })
                    })
            })
            .partition_result::<Vec<_>, Vec<_>, _, _>()
            .pipe(|(oks, errs)| {
                errors.extend(errs);
                oks
            });

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            root: config.root,
            links,
            files,
        })
    }
}

/// Helper to return a single error from a function that returns a `Vec<Error>`
///
/// Useful for **unrecoverable** errors
fn single_err(err: impl Into<Error>) -> Vec<Error> {
    vec![err.into()]
}
