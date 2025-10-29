use std::env::current_dir;
use std::path::PathBuf;
use std::sync::LazyLock;

use assert_cmd::Command;

pub(crate) type TestResult = anyhow::Result<()>;

pub(crate) fn data_dir() -> &'static PathBuf {
    static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        current_dir()
            .unwrap()
            .join("../../tests/data")
            .canonicalize()
            .unwrap()
            .to_path_buf()
    });

    &DATA_DIR
}

pub(crate) fn pica_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("pica"))
}
