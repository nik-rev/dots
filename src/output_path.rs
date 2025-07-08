//! Contains [`OutputPath`]

use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use etcetera::BaseStrategy as _;
use eyre::{Context as _, eyre};
use interpolator::Formattable;
use tap::Pipe as _;

use crate::stdx::PathExt as _;

/// Represents a path that will be written to.
///
/// The `FromStr` impl for this allow for interpolation, i.e.
/// if the config directory is `~/.config`, then `{config}/helix` will
/// parse as `~/.config/helix`
#[nutype::nutype(derive(AsRef, Clone))]
pub struct OutputPath(PathBuf);

impl Display for OutputPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref().show())
    }
}

impl FromStr for OutputPath {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let strategy = etcetera::choose_base_strategy()
            .with_context(|| eyre!("failed to obtain base strategy"))?;

        [
            (
                "config",
                strategy
                    .config_dir()
                    .to_string_lossy()
                    .to_string()
                    .pipe_ref(Formattable::display),
            ),
            (
                "home",
                strategy
                    .home_dir()
                    .to_string_lossy()
                    .to_string()
                    .pipe_ref(Formattable::display),
            ),
        ]
        .pipe(HashMap::from)
        .pipe(|hm| interpolator::format(s, &hm))
        .with_context(|| eyre!("failed to parse marker for: {s}"))
        .map(PathBuf::from)
        .map(OutputPath::new)
    }
}

impl<'de> serde::Deserialize<'de> for OutputPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<Self>()
            .map_err(serde::de::Error::custom)
    }
}
