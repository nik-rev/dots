//! Contains [`OutputPath`]

use std::{fmt::Display, path::PathBuf, str::FromStr};

use etcetera::BaseStrategy as _;
use eyre::{Context as _, eyre};

use crate::stdx::PathExt as _;

/// Represents a path that will be written to.
///
/// The `FromStr` impl for this allow for interpolation, i.e.
/// if the config directory is `~/.config`, then `{config}/helix` will
/// parse as `~/.config/helix`
#[nutype::nutype(derive(AsRef, Clone, Debug, From, PartialEq))]
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

        // expand tilde: ~/foo -> /home/user/foo
        let s = if let Some(s) = s.strip_prefix("~/") {
            strategy.home_dir().join(s).to_string_lossy().to_string()
        } else {
            s.to_string()
        };

        let mut chars = s.chars();
        let mut total = String::new();

        while let Some(ch) = chars.next() {
            if ch != '{' {
                total.push(ch);
                continue;
            }

            // if it's '{', now everything inside is a variable
            let mut variable = String::new();
            while let Some(ch) = chars.next()
                && ch != '}'
            {
                variable.push(ch);
            }

            let path = match variable.as_str() {
                "data_dir" => strategy.data_dir(),
                "config_dir" => strategy.config_dir(),
                "cache_dir" => strategy.cache_dir(),
                s if s.starts_with('$') => {
                    let env = s.strip_prefix("$").expect("it starts with `$`");
                    let Ok(var) = std::env::var(env) else {
                        continue;
                    };
                    var.into()
                }
                var => {
                    log::warn!("unknown variable: {var}");
                    continue;
                }
            };
            let path = path.to_string_lossy().to_string();

            total.push_str(&path);
        }

        Ok(OutputPath::new(total.into()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            "{config_dir}".parse::<OutputPath>().unwrap(),
            etcetera::choose_base_strategy()
                .unwrap()
                .config_dir()
                .into()
        );
    }
}
