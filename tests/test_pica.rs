use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

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

static INVALID: &str = include_str!("data/invalid.dat");
static SAMPLE1: &str = include_str!("data/1.dat");
static SAMPLE2: &str = include_str!("data/2.dat");
static SAMPLE3: &str = include_str!("data/3.dat");
static SAMPLE4: &str = include_str!("data/4.dat");
static SAMPLE5: &str = include_str!("data/5.dat");

#[test]
fn cat_command() {
    let result = CliRunner::new().invoke("cat", &["tests/data/1.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    let result = CliRunner::new()
        .invoke("cat", &["tests/data/1.dat", "tests/data/2.dat"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "cat",
        &[
            "--skip-invalid",
            "tests/data/1.dat",
            "tests/data/invalid.dat",
            "tests/data/empty.dat",
            "tests/data/2.dat",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );

    let result = CliRunner::new()
        .invoke("cat", &["tests/data/1.dat", "tests/data/invalid.dat"]);
    assert!(!result.status.success());
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
fn filter_command() {
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "012A/*.a == '1'", "tests/data/5.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE5);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 != '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "003@.0 =^ '123'", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "002@.0 =$ 'p2'", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@.0 =~ '^Tp[12]$'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@.0 in ['Tp1', 'Tp3']",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 in ['Tp1', 'Tp3']}",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 == 'T\n1\\ ' || 0 == 'Tp1' || 0 == 'Tp3'}",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 == 'Tp1' || 0 == 'Tp3'} || 003@.0 == '234567891\u{0058}'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 =^ 'Tp' && 0 =$ '2'}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0 =^ 'Tp' && 0 =$ '2'} && 003@.0 == '234567891X'",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{ (0 =^ 'Tp' && 0 =$ '2') || 0 == 'Tp1' }",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 =$ 'X' && (002@{0 =^ 'Tp' && 0 =$ '2'} || 002@.0 == 'Tp1')",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{!(0 == 'Tp2' || c?)}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "!(002@.0 == 'Tp2' || 002@.0 == 'Tp3')",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "002@.0?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "002@{0? && 0 == 'Tp2'}",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE2);

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "012A/00?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &["--skip-invalid", "013B?", "tests/data/all.dat.gz"],
    );

    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty(),);

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--invert-match",
            "003@.0 == '123456789X'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new()
        .invoke("filter", &["003@.! == '0123456789X'", "tests/data/1.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new().invoke(
        "filter",
        &["003@.0 == '0123456789X'", "tests/data/invalid.dat"],
    );
    assert!(!result.status.success());

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "003@.0 == '123456789X'",
            "tests/data/1.dat",
        ],
    );

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    // limit
    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--limit",
            "1",
            "002@.0 =^ 'T'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", SAMPLE1)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--limit",
            "2",
            "002@.0 =^ 'T'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--limit",
            "999",
            "002@.0 =^ 'T'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--limit",
            "0",
            "002@.0 =^ 'T'",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3)
    );

    let result = CliRunner::new().invoke(
        "filter",
        &[
            "--skip-invalid",
            "--limit",
            "abc",
            "002@.0 =^ 'T'",
            "tests/data/all.dat.gz",
        ],
    );
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

    // threshold
    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "-t", "2", "002@.0", "tests/data/dump.dat"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "Tp1,2\n");

    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "-t", "100", "002@.0", "tests/data/dump.dat"],
    );
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());

    let result = CliRunner::new().invoke(
        "frequency",
        &["-s", "-t", "abc", "002@.0", "tests/data/dump.dat"],
    );
    assert_eq!(result.status.success(), false);

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
    let result =
        CliRunner::new().invoke("invalid", &["tests/data/invalid.dat"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    let result = CliRunner::new().invoke("invalid", &["tests/data/all.dat.gz"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), INVALID);

    let result = CliRunner::new().invoke("invalid", &["tests/data/1.dat"]);
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());
}

#[test]
fn test_json_cmd() {
    static SAMPLE1_JSON: &str = "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"123456789X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"Tp1\"}]},{\"name\":\"012A\",\"occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\":\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

    static SAMPLE2_JSON: &str = "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"234567891X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"Tp2\"}]},{\"name\":\"012A\",\"occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\":\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

    static SAMPLE3_JSON: &str = "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"345678912X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"Tp1\"}]},{\"name\":\"012A\",\"occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\":\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

    static SAMPLE4_JSON: &str = "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\":\"0\",\"value\":\"33445566X\"}]},{\"name\":\"012A\",\"occurrence\":null,\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\":\"b\",\"value\":\"1\"}]},{\"name\":\"012A\",\"occurrence\":null,\"subfields\":[{\"name\":\"a\",\"value\":\"2\"},{\"name\":\"a\",\"value\":\"3\"}]}]}";

    let result = CliRunner::new()
        .invoke("json", &["--skip-invalid", "tests/data/empty.dat"]);
    assert!(result.status.success());

    assert_eq!(String::from_utf8(result.stdout).unwrap(), "[]");

    let result = CliRunner::new()
        .invoke("json", &["--skip-invalid", "tests/data/1.dat"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("[{}]", SAMPLE1_JSON)
    );

    let result = CliRunner::new()
        .invoke("json", &["--skip-invalid", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!(
            "[{},{},{},{}]",
            SAMPLE1_JSON, SAMPLE2_JSON, SAMPLE3_JSON, SAMPLE4_JSON
        )
    );

    // invalid
    let result = CliRunner::new().invoke("json", &["tests/data/invalid.dat"]);
    assert!(!result.status.success());
}

#[test]
fn partition_command() {
    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "partition",
        &[
            "--skip-invalid",
            "--outdir",
            outdir.to_str().unwrap(),
            "002@.0",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("Tp1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE3));

    let content = std::fs::read_to_string(outdir.join("Tp2.dat")).unwrap();
    assert_eq!(content, SAMPLE2);

    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path().join("part-test");

    let result = CliRunner::new().invoke(
        "partition",
        &[
            "--skip-invalid",
            "--outdir",
            outdir.to_str().unwrap(),
            "002@.0",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("Tp1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE3));

    let content = std::fs::read_to_string(outdir.join("Tp2.dat")).unwrap();
    assert_eq!(content, SAMPLE2);

    let result = CliRunner::new()
        .invoke("partition", &["002@.0", "tests/data/invalid.dat"]);

    assert!(!result.status.success());
}

#[test]
fn print_command() {
    let result = CliRunner::new().invoke("print", &["tests/data/1.dat"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "003@ $0 123456789X\n002@ $0 Tp1\n012A/00 $a 1 $a 2 $b 1\n\n"
    );

    let result = CliRunner::new().invoke("print", &["tests/data/invalid.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("print", &["--skip-invalid", "tests/data/invalid.dat"]);

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "");

    let result = CliRunner::new()
        .invoke("print", &["--skip-invalid", "tests/data/empty.dat"]);

    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "");
}

#[test]
fn sample_command() {
    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "1", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    let output = String::from_utf8(result.stdout).unwrap();
    assert!(
        output == SAMPLE1
            || output == SAMPLE2
            || output == SAMPLE3
            || output == SAMPLE4
    );

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "2", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    let output = String::from_utf8(result.stdout).unwrap();
    assert!(
        output == format!("{}{}", SAMPLE1, SAMPLE2)
            || output == format!("{}{}", SAMPLE1, SAMPLE3)
            || output == format!("{}{}", SAMPLE1, SAMPLE4)
            || output == format!("{}{}", SAMPLE2, SAMPLE1)
            || output == format!("{}{}", SAMPLE2, SAMPLE3)
            || output == format!("{}{}", SAMPLE2, SAMPLE4)
            || output == format!("{}{}", SAMPLE3, SAMPLE1)
            || output == format!("{}{}", SAMPLE3, SAMPLE2)
            || output == format!("{}{}", SAMPLE3, SAMPLE4)
            || output == format!("{}{}", SAMPLE4, SAMPLE1)
            || output == format!("{}{}", SAMPLE4, SAMPLE2)
            || output == format!("{}{}", SAMPLE4, SAMPLE3)
    );

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "100", "tests/data/1.dat"]);
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE1);

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "0", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "-1", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("sample", &["--skip-invalid", "a", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());

    let result =
        CliRunner::new().invoke("sample", &["1", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());
}

#[test]
fn select_command() {
    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "003@.0,002@.0", "tests/data/1.dat"],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "123456789X,Tp1\n"
    );

    let result = CliRunner::new().invoke(
        "select",
        &[
            "--skip-invalid",
            "--tsv",
            "003@.0,002@.0",
            "tests/data/1.dat",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "123456789X\tTp1\n"
    );

    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "003@.0,012A/00.a", "tests/data/2.dat"],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        concat!("234567891X,1\n", "234567891X,2\n")
    );

    let result = CliRunner::new().invoke(
        "select",
        &["--skip-invalid", "013B.a,013B/00.c", "tests/data/2.dat"],
    );
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout).unwrap().is_empty());

    // filter
    let result = CliRunner::new().invoke(
        "select",
        &[
            "--skip-invalid",
            "003@.0,012A/*{b == '1', a}",
            "tests/data/4.dat",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "33445566X,1\n".to_string()
    );

    let result =
        CliRunner::new().invoke("select", &["003!.0", "tests/data/1.dat"]);
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("select", &["003@.0", "tests/data/invalid.dat"]);
    assert!(!result.status.success());

    // header
    let result = CliRunner::new().invoke(
        "select",
        &[
            "--skip-invalid",
            "--header",
            "idn,bbg",
            "003@.0,002@.0",
            "tests/data/1.dat",
        ],
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "idn,bbg\n123456789X,Tp1\n"
    );
}

#[test]
fn slice_command() {
    let result =
        CliRunner::new().invoke("slice", &["-s", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}{}", SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--start", "1", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new()
        .invoke("slice", &["-s", "--end", "2", "tests/data/all.dat.gz"]);
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE1, SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &["-s", "--start", "1", "--end", "2", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}", SAMPLE2)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "1",
            "--length",
            "4",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}{}", SAMPLE2, SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &["-s", "--start", "2", "--end", "4", "tests/data/all.dat.gz"],
    );
    assert!(result.status.success());
    assert_eq!(String::from_utf8(result.stdout).unwrap(), SAMPLE3);

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "2",
            "--length",
            "2",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(result.status.success());

    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{}{}", SAMPLE3, SAMPLE4)
    );

    let result = CliRunner::new().invoke(
        "slice",
        &[
            "-s",
            "--start",
            "1",
            "--end",
            "2",
            "--length",
            "1",
            "tests/data/all.dat.gz",
        ],
    );
    assert!(!result.status.success());

    let result = CliRunner::new()
        .invoke("slice", &["--start", "2", "tests/data/all.dat.gz"]);
    assert!(!result.status.success());
}

#[test]
fn test_split_command() {
    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "split",
        &[
            "--skip-invalid",
            "--template",
            "SPLIT_{}.dat",
            "--outdir",
            outdir.to_str().unwrap(),
            "2",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("SPLIT_0.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE2));

    let content = std::fs::read_to_string(outdir.join("SPLIT_1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE3, SAMPLE4));

    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path().join("split-test");

    let result = CliRunner::new().invoke(
        "split",
        &[
            "--skip-invalid",
            "--template",
            "SPLIT_{}.dat",
            "--outdir",
            outdir.to_str().unwrap(),
            "2",
            "tests/data/all.dat.gz",
        ],
    );

    assert!(result.status.success());

    let content = std::fs::read_to_string(outdir.join("SPLIT_0.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE1, SAMPLE2));

    let content = std::fs::read_to_string(outdir.join("SPLIT_1.dat")).unwrap();
    assert_eq!(content, format!("{}{}", SAMPLE3, SAMPLE4));

    let result =
        CliRunner::new().invoke("split", &["0", "tests/data/invalid.dat"]);
    assert!(!result.status.success());

    let result =
        CliRunner::new().invoke("split", &["a", "tests/data/invalid.dat"]);
    assert!(!result.status.success());

    let tempdir = TempDir::new().unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "split",
        &[
            "1",
            "--outdir",
            outdir.to_str().unwrap(),
            "tests/data/invalid.dat",
        ],
    );
    assert!(!result.status.success());
}
