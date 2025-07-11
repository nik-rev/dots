//! Extensions to the standard library

use eyre::{Context as _, Result, eyre};
use simply_colored::*;
use std::{
    iter,
    path::{Path, PathBuf},
};

/// Extension trait for [`Path`]
#[easy_ext::ext(PathExt)]
pub impl<T: AsRef<Path>> T {
    /// Show the colored path
    #[allow(clippy::disallowed_methods, reason = "definition of `show_path`")]
    fn show(&self) -> String {
        format!("{CYAN}{}{RESET}", self.as_ref().display())
    }

    /// Like [`Path::strip_prefix`], but includes an informative error message
    fn strip_prefix(&self, prefix: impl AsRef<Path>) -> Result<&Path> {
        self.as_ref().strip_prefix(&prefix).with_context(|| {
            eyre!(
                "failed to strip prefix {} from {}",
                prefix.show(),
                self.show()
            )
        })
    }
}

/// Traverses all directories upwards from the `base_dir`
///
/// For example, if `base_dir` is `/home/user/project/name/`, then the iterator yields:
/// - `/home/user/project/name/`
/// - `/home/user/project/`
/// - `/home/user/`
/// - `/home/`
/// - `/`
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
