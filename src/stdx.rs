//! Extensions to the standard library

use eyre::{Context as _, ContextCompat as _, Result, eyre};
use simply_colored::*;
use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

/// Extension trait for [`Iterator`]
pub trait IteratorExt: Iterator {
    /// Collect all of the `Ok(T)` variants into a collection of `T`s
    /// If any `Err(E)` is found, collect all errors in a collection of `E`s
    fn try_collect_all<Okays, Errors, Okay, Error>(mut self) -> Result<Okays, Errors>
    where
        Self: Iterator<Item = Result<Okay, Error>> + Sized,
        Okays: Extend<Okay> + Default,
        Errors: FromIterator<Error>,
    {
        let mut oks = Okays::default();
        while let Some(res) = self.next() {
            match res {
                Ok(ok) => oks.extend(iter::once(ok)),
                Err(err) => {
                    let err = iter::once(err)
                        .chain(self.filter_map(Result::err))
                        .collect::<Errors>();
                    return Err(err);
                }
            }
        }
        Ok(oks)
    }

    /// Collect all of the `Ok(T)` variants into a `Ok(Vec<T>)`
    /// If any `Err(E)` is found, collect all errors in a `Err(Vec<E>)`
    fn try_collect_all_vec<Okay, Error>(self) -> Result<Vec<Okay>, Vec<Error>>
    where
        Self: Iterator<Item = Result<Okay, Error>> + Sized + ExactSizeIterator,
    {
        self.try_collect_all::<Vec<_>, Vec<_>, _, _>()
    }
}

impl<A> IteratorExt for A where A: Iterator {}

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

/// Write given `contents` to the given `path`.
///
/// - If a file already exists at that location, overwrite it
/// - If we are trying to write to a path that does not have a parent directory,
///   create the parent directory
pub fn write_file(path: impl AsRef<Path>, contents: &impl ToString) -> Result<()> {
    let path = path.as_ref();
    let contents = contents.to_string();

    remove_file(path).with_context(|| eyre!("failed to remove file: {}", path.show()))?;

    log::warn!("{RED}removed{RESET} {}", path.show());

    let dir = path
        .parent()
        .with_context(|| eyre!("failed to obtain parent of {}", path.show()))?;

    // 2. Create parent directory which will contain the file downloaded from the link
    fs::create_dir_all(dir)
        .with_context(|| eyre!("failed to create directory for {}", dir.show()))?;

    fs::write(path, contents).with_context(|| eyre!("failed to write to {}", path.show()))?;

    log::info!("wrote to {}", path.show());

    Ok(())
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
    fn try_collect_all() {
        assert_eq!(
            vec![Ok(1), Err("1"), Ok(2), Err("2"), Err("3")]
                .into_iter()
                .try_collect_all_vec(),
            Err(vec!["1", "2", "3"])
        );
        assert_eq!(
            vec![Ok::<_, ()>(1), Ok(1), Ok(2), Ok(2), Ok(3)]
                .into_iter()
                .try_collect_all_vec(),
            Ok(vec![1, 1, 2, 2, 3])
        );
        assert_eq!(
            vec![].into_iter().try_collect_all_vec::<(), ()>(),
            Ok(vec![])
        );
    }

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
