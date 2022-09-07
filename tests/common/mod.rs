use std::fs::File;
use std::io::Write;

use assert_cmd::Command;
use tempfile::{Builder, TempDir};

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(crate) struct TestContext {
    pub(crate) tempdir: TempDir,
}

impl TestContext {
    pub(crate) fn new() -> Self {
        let tempdir = Builder::new().tempdir().unwrap();
        TestContext { tempdir }
    }
}

pub(crate) trait CommandExt {
    fn with_config(
        &mut self,
        ctx: &TestContext,
        content: &str,
    ) -> &mut Self;
}

impl CommandExt for Command {
    fn with_config(
        &mut self,
        ctx: &TestContext,
        content: &str,
    ) -> &mut Self {
        let filename = ctx.tempdir.path().join("Pica.toml");
        let mut config =
            File::create(&filename).expect("create config file");
        config.write_all(content.as_bytes()).expect("write config");
        config.flush().expect("flush config");

        self.arg("--config").arg(filename);
        self
    }
}
