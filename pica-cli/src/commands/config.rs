use std::process::ExitCode;

use clap::Parser;

use crate::error::{bail, CliError, CliResult};

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
fn print_option<T>(key: &str, value: Option<T>)
where
    T: ToString,
{
    println!(
        "{key} = {}",
        match value {
            Some(value) => value.to_string(),
            None => "None".to_string(),
        }
    );
}

impl Config {
    pub(crate) fn execute(self) -> CliResult {
        let mut config = crate::config::Config::discover()?;
        let name = match self.name.as_str() {
            name if name == "skip-invalid" => name,
            name => {
                bail!("unknown config option `{name}`");
            }
        };

        if self.value.is_some() {
            let value = self.value.unwrap();
            match name {
                "skip-invalid" => {
                    if let Ok(value) = value.parse::<bool>() {
                        config.skip_invalid = value;
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
                _ => unreachable!(),
            }

            config.save()?;
        } else if self.get || (!self.unset && !self.set) {
            match name {
                "skip-invalid" => {
                    print_option(name, Some(&config.skip_invalid));
                }
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }

        Ok(ExitCode::SUCCESS)
    }
}
