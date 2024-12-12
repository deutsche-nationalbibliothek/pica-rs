use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Command, Parser};
use clap_complete::{generate, Shell};

use crate::error::CliResult;

/// Generate shell completions (e.g. Bash or ZSH)
#[derive(Parser, Debug)]
pub(crate) struct Completions {
    /// Output the shell completion file for the given shell.
    shell: Shell,

    /// Write output to FILENAME instead of stdout
    #[arg(short, long, value_name = "FILENAME")]
    output: Option<OsString>,
}

impl Completions {
    pub(crate) fn execute(self, cmd: &mut Command) -> CliResult {
        use Shell::*;

        let mut wtr: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout().lock()),
        };

        match self.shell {
            Bash => generate(Bash, cmd, "pica", &mut wtr),
            Elvish => generate(Elvish, cmd, "pica", &mut wtr),
            Fish => generate(Fish, cmd, "pica", &mut wtr),
            PowerShell => generate(PowerShell, cmd, "pica", &mut wtr),
            Zsh => generate(Zsh, cmd, "pica", &mut wtr),
            _ => unreachable!(),
        }

        wtr.flush()?;

        Ok(ExitCode::SUCCESS)
    }
}
