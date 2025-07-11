#![cfg(test)]
//! tests

// use std::path::{Path, PathBuf};

// use clap::Parser as _;
// use dots::{Cli, Config};

// struct Item {
//     path: PathBuf,
//     contents: Option<String>,
// }

// impl From<PathBuf> for Item {
//     fn from(value: PathBuf) -> Self {
//         Item {
//             path: value,
//             contents: None,
//         }
//     }
// }

// impl From<(PathBuf, String)> for Item {
//     fn from((path, contents): (PathBuf, String)) -> Self {
//         Item {
//             path,
//             contents: Some(contents),
//         }
//     }
// }

// fn run<'a, 'b>(
//     cwd: &Path,
//     // A list of paths to create
//     paths: impl IntoIterator<Item = impl Into<Item<'a, 'b>>>,
//     args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
// ) {
//     let cli = Cli::try_parse_from(args).unwrap();
//     let conf = Config::find(cwd);
//     let paths: Vec<Item> = paths.into_iter().map(Into::into).collect();
// }

// #[test]
// fn hmm() {
//     let tmp = tempfile::tempdir().unwrap();
//     let tmp = tmp.path();

//     run(tmp, [("dots.toml", "foobar")], "--help");

//     let conf = tmp.join("dots.toml");
// }
