use std::process::ExitCode;

use clap::Parser;

use crate::prelude::*;

/// Get and set configuration options.
#[derive(Debug, Parser)]
pub(crate) struct Config {
    /// Get the value for the given key.
    #[arg(long, conflicts_with_all = ["value", "unset", "set"])]
    get: bool,

    /// Remove the key from the config.
    #[arg(long, conflicts_with_all = ["value", "get", "set"])]
    unset: bool,

    /// Set the value for the given key.
    #[arg(long, requires = "value", conflicts_with_all = ["get", "unset"])]
    set: bool,

    /// The name of the config option.
    name: String,

    /// The (new) value of the config option.
    #[arg(conflicts_with_all = ["get", "unset"])]
    value: Option<String>,
}

#[inline]
fn print_option<T: ToString>(value: Option<T>) {
    println!(
        "{}",
        match value {
            Some(value) => value.to_string(),
            None => "None".to_string(),
        }
    );
}

impl Config {
    pub(crate) fn execute(
        self,
        config: &mut crate::config::Config,
    ) -> CliResult {
        let name = match self.name.as_str() {
            name if name == "skip-invalid" => name,
            name if name == "normalization" => name,
            name => {
                bail!("unknown config option `{name}`");
            }
        };

        if let Some(value) = self.value {
            match name {
                "skip-invalid" => {
                    if let Ok(value) = value.parse::<bool>() {
                        config.skip_invalid = value;
                    } else {
                        bail!("invalid value `{value}`");
                    }
                }
                "normalization" => {
                    if let Ok(value) =
                        value.parse::<NormalizationForm>()
                    {
                        config.normalization = Some(value);
                    } else {
                        bail!("invalid value `{value}`");
                    }
                }
                _ => unreachable!(),
            }

            config.save()?;
        } else if self.unset {
            match name {
                "skip-invalid" => {
                    config.skip_invalid = false;
                }
                "normalization" => {
                    config.normalization = None;
                }
                _ => unreachable!(),
            }

            config.save()?;
        } else if self.get || (!self.unset && !self.set) {
            match name {
                "skip-invalid" => {
                    print_option(Some(&config.skip_invalid));
                }
                "normalization" => {
                    print_option(config.normalization.as_ref())
                }
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }

        Ok(ExitCode::SUCCESS)
    }
}
