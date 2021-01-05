use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug)]
pub struct CliRunner<'a> {
    root_dir: &'a Path,
    pica_bin: PathBuf,
}

impl<'a> CliRunner<'a> {
    pub fn new() -> Self {
        let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let pica_bin = root_dir.join("target/debug/pica");

        CliRunner { root_dir, pica_bin }
    }

    pub fn invoke<I, S>(&self, cmd: &str, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(&self.root_dir.join("target/debug/pica"))
            .current_dir(self.root_dir)
            .arg(cmd)
            .args(args)
            .output()
            .unwrap()
    }
}
