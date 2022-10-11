use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};

use clap::{Command, Parser};
use clap_complete::{generate, Shell};

use crate::util::CliResult;

#[derive(Parser, Debug)]
pub(crate) struct Completions {
    shell: Shell,

    /// Write output to <OUTPUT> instead of stdout
    #[arg(short, long)]
    output: Option<OsString>,
}

impl Completions {
    pub(crate) fn run(self, cmd: &mut Command) -> CliResult<()> {
        let mut writer: Box<dyn Write> = match self.output {
            Some(filename) => Box::new(File::create(filename)?),
            None => Box::new(io::stdout()),
        };

        match self.shell {
            Shell::Bash => {
                generate(Shell::Bash, cmd, "pica", &mut writer)
            }
            Shell::Elvish => {
                generate(Shell::Elvish, cmd, "pica", &mut writer)
            }
            Shell::Fish => {
                generate(Shell::Fish, cmd, "pica", &mut writer)
            }
            Shell::PowerShell => {
                generate(Shell::PowerShell, cmd, "pica", &mut writer)
            }
            Shell::Zsh => {
                generate(Shell::Zsh, cmd, "pica", &mut writer)
            }
            _ => unreachable!(),
        }

        writer.flush()?;
        Ok(())
    }
}

// use clap::Arg;
// use clap_complete::{generate, Shell};

// use crate::util::{CliArgs, CliResult, Command};
