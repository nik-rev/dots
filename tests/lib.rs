#![cfg(test)]
//! tests

use etcetera::BaseStrategy as _;
use pretty_assertions::assert_eq;

use std::{collections::HashSet, fs, path::Path};

use dots::{World, WritePath, process};
use tap::Pipe as _;
use tempfile::tempdir;

/// hum
///
/// # Panics
///
/// ?
#[track_caller]
pub fn check(
    cwd: impl AsRef<Path>,
    paths: impl IntoIterator<Item = (impl AsRef<Path>, impl AsRef<str>)>,
) {
    // NOTE: we use a HashSet internally for the test because we don't care about the order in which paths get written
    let writes = World::new(cwd.as_ref())
        .unwrap()
        .pipe(process)
        .unwrap()
        .writes
        .into_iter()
        .pipe(HashSet::from_iter);

    assert_eq!(
        writes,
        paths
            .into_iter()
            .map(|(path, contents)| WritePath {
                path: path.as_ref().to_path_buf(),
                contents: contents.as_ref().to_string()
            })
            .collect::<HashSet<_>>()
    );
}

fn create_files_in(
    dir: &Path,
    files: impl IntoIterator<Item = (impl AsRef<Path>, impl AsRef<str>)>,
) {
    for (path, contents) in files {
        let path = dir.join(path.as_ref());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents.as_ref()).unwrap();
    }
}

#[test]
fn it_works() {
    let dir = tempdir().unwrap();
    let dir = dir.path();

    let config_dir = etcetera::choose_base_strategy().unwrap().config_dir();

    create_files_in(
        dir,
        [
            (
                "dots.toml",
                r#"
                [[dir]]
                input = "configs"
                output = "{config}"
                "#,
            ),
            ("configs/foo.txt", "foo"),
            ("configs/bar.txt", "bar"),
        ],
    );

    check(
        dir,
        [
            (config_dir.join("foo.txt"), "foo"),
            (config_dir.join("bar.txt"), "bar"),
        ],
    );
}
