use std::ffi::OsStr;
use std::fs::read_to_string;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

static INVALID: &'static str = include_str!("data/invalid.dat");
static R12283643X: &'static str = include_str!("data/12283643X.dat");
static R119232022: &'static str = include_str!("data/119232022.dat");

use pica::{ReaderBuilder, StringRecord};

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

#[test]
fn cat_command() {
    let result = CliRunner::new().invoke(
        "cat",
        &["tests/data/12283643X.dat", "tests/data/119232022.dat.gz"],
    );
    assert!(result.status.success());

    let mut reader =
        ReaderBuilder::new().from_reader(Cursor::new(result.stdout));
    let mut iter = reader.records();

    let record: StringRecord = iter.next().unwrap().unwrap();
    assert_eq!(record, StringRecord::from_bytes(R12283643X).unwrap());

    let record: StringRecord = iter.next().unwrap().unwrap();
    assert_eq!(record, StringRecord::from_bytes(R119232022).unwrap());

    assert!(iter.next().is_none());

    let result = CliRunner::new().invoke(
        "cat",
        &["tests/data/12283643X.dat", "tests/data/invalid.dat"],
    );
    assert_eq!(result.status.success(), false);

    let result = CliRunner::new().invoke(
        "cat",
        &[
            "--skip-invalid",
            "tests/data/12283643X.dat",
            "tests/data/invalid.dat",
        ],
    );
    assert!(result.status.success());
}

#[test]
fn completion_command() {
    for shell in ["fish", "bash", "zsh"].iter() {
        let result = CliRunner::new().invoke("completion", &[shell]);
        assert!(result.status.success());
        assert_eq!(String::from_utf8(result.stdout).unwrap().is_empty(), false);
    }

    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "completion",
        &[
            "-o",
            outdir.join("completion.bash").to_str().unwrap(),
            "bash",
        ],
    );
    assert!(result.status.success());
    assert!(outdir.join("completion.bash").exists());

    // invalid shell
    let result = CliRunner::new().invoke("completion", &["powershell"]);
    assert_eq!(result.status.success(), false);
}

#[test]
fn frequency_command() {
    let result = CliRunner::new()
        .invoke("frequency", &["002@.0", "tests/data/119232022.dat.gz"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,1\n");

    // invalid
    let result = CliRunner::new()
        .invoke("frequency", &["002@.0", "tests/data/dump.dat"]);
    assert_eq!(result.status.success(), false);

    // skip-invalid
    let result = CliRunner::new()
        .invoke("frequency", &["-s", "002@.0", "tests/data/dump.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\nTs1,1\n");

    // invalid limit
    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "--limit", "abc", "002@.0", "tests/data/dump.dat"],
    );
    assert_eq!(result.status.success(), false);

    // limit
    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "--limit", "1", "002@.0", "tests/data/dump.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\n");

    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "-l", "100", "002@.0", "tests/data/dump.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\nTs1,1\n");

    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "-l", "0", "002@.0", "tests/data/dump.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\nTs1,1\n");

    // output
    let tempdir = TempDir::new().unwrap();
    let filename = tempdir.path().join("frequency.csv");

    let result = CliRunner::new().invoke(
        "frequency",
        &[
            "-s",
            "002@.0",
            "--output",
            filename.to_str().unwrap(),
            "tests/data/dump.dat",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        read_to_string(filename.to_str().unwrap()).unwrap(),
        "Tp1,2\nTs1,1\n"
    );
}

#[test]
fn invalid_command() {
    // filter single invalid record
    let result =
        CliRunner::new().invoke("invalid", &["tests/data/invalid.dat"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    // valid record are not filtered
    let result =
        CliRunner::new().invoke("invalid", &["tests/data/12283643X.dat"]);
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());

    // filter invalid records from dump
    let result = CliRunner::new().invoke("invalid", &["tests/data/dump.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    // test write output to file
    let tempdir = TempDir::new().unwrap();
    let filename = tempdir.path().join("invalid.dat");

    let result = CliRunner::new().invoke(
        "invalid",
        &[
            "tests/data/dump.dat",
            "--output",
            filename.to_str().unwrap(),
        ],
    );
    assert!(result.status.success());

    let mut reader = ReaderBuilder::new()
        .skip_invalid(false)
        .from_path(filename)
        .unwrap();

    assert_eq!(reader.records().count(), 1);
}
