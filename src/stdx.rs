//! Extensions to the standard library

use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

/// Traverses all directories upwards from the `base_dir`
///
/// For example, if `base_dir` is `/home/user/project/name/`, then the iterator yields:
/// - `/home/user/project/name/`
/// - `/home/user/project/`
/// - `/home/user/`
/// - `/home/`
/// - `/`
#[allow(unused, reason = "used later")]
pub fn traverse_upwards(base_dir: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    let mut current_dir = Some(base_dir.as_ref().to_path_buf());
    iter::once(base_dir.as_ref().to_path_buf()).chain(iter::from_fn(move || {
        if let Some(d) = &current_dir {
            current_dir = d.parent().map(Path::to_path_buf);
            current_dir.clone()
        } else {
            None
        }
    }))
}

/// Remove a file, and if it's not found then that is not considered an error
pub fn remove_file(file: impl AsRef<Path>) -> Result<(), io::Error> {
    match fs::remove_file(file) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
        Ok(()) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools as _;

    use super::*;

    #[test]
    fn traverse_upwards() {
        let path = PathBuf::from("/home/user/project/name/");

        assert_eq!(
            super::traverse_upwards(path).collect_vec(),
            vec![
                PathBuf::from("/home/user/project/name/"),
                PathBuf::from("/home/user/project/"),
                PathBuf::from("/home/user/"),
                PathBuf::from("/home/"),
                PathBuf::from("/"),
            ]
        );
    }
}
