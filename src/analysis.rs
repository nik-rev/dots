//! Contains [`Analysis`]

use std::path::PathBuf;
use std::{fs, io};

use simply_colored::*;

use crate::PathExt as _;

/// Write contents to the path
#[derive(Debug)]
pub struct WritePath {
    /// Path to write
    pub path: PathBuf,
    /// What to write
    pub contents: String,
}

/// Analysis represents finished computation
#[derive(Debug)]
pub struct Analysis {
    /// A list of paths to write
    pub writes: Vec<WritePath>,
}

impl Analysis {
    /// Finish the analysis
    pub fn finish(self) {
        for WritePath { path, contents } in self.writes {
            let contents = contents.to_string();

            if let Err(err) = match fs::remove_file(&path) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(err) => Err(err),
                Ok(()) => Ok(()),
            } {
                log::error!("failed to remove file {}: {err}", path.show());
                continue;
            }

            log::warn!("{RED}removed{RESET} {}", path.show());

            let Some(dir) = path.parent() else {
                log::error!("failed to obtain parent of {}", path.show());
                continue;
            };

            // 2. Create parent directory which will contain the file downloaded from the link
            if let Err(err) = fs::create_dir_all(dir) {
                log::error!("failed to create directory for {}: {err}", dir.show());
            }

            if let Err(err) = fs::write(&path, contents) {
                log::error!("failed to write to {}: {err}", path.show());
            }

            log::info!("wrote to {}", path.show());
        }
    }
}
