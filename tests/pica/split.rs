use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, remove_file, File};
use std::io::Read;
use tempfile::Builder;

use crate::common::{CommandExt, TestContext, TestResult};

#[test]
fn pica_split_default() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat", "tests/data/1004916019.dat"),
        ("1.dat", "tests/data/119232022.dat"),
        ("2.dat", "tests/data/000008672.dat"),
        ("3.dat", "tests/data/000016586.dat"),
        ("4.dat", "tests/data/000016756.dat"),
        ("5.dat", "tests/data/000009229.dat"),
        ("6.dat", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(filename).unwrap(),
            read_to_string(sample).unwrap()
        );

        remove_file(filename).unwrap();
    }

    Ok(())
}

#[test]
fn pica_split_multiple_files() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .arg("tests/data/119232022.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat", "tests/data/1004916019.dat"),
        ("1.dat", "tests/data/119232022.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    let data = read_to_string("tests/data/119232022.dat")?;
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/1004916019.dat")
        .arg("-")
        .write_stdin(data)
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat", "tests/data/1004916019.dat"),
        ("1.dat", "tests/data/119232022.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    Ok(())
}

#[test]
fn pica_split_stdin() -> TestResult {
    let data = read_to_string("tests/data/119232022.dat")?;
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .write_stdin(data)
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [("0.dat", "tests/data/119232022.dat")];
    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    let data = read_to_string("tests/data/119232022.dat")?;
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("-")
        .write_stdin(data)
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [("0.dat", "tests/data/119232022.dat")];
    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    Ok(())
}

#[test]
fn pica_split_outdir() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat", "tests/data/1004916019.dat"),
        ("1.dat", "tests/data/119232022.dat"),
        ("2.dat", "tests/data/000008672.dat"),
        ("3.dat", "tests/data/000016586.dat"),
        ("4.dat", "tests/data/000016756.dat"),
        ("5.dat", "tests/data/000009229.dat"),
        ("6.dat", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir.join("foo"))
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat", "tests/data/1004916019.dat"),
        ("1.dat", "tests/data/119232022.dat"),
        ("2.dat", "tests/data/000008672.dat"),
        ("3.dat", "tests/data/000016586.dat"),
        ("4.dat", "tests/data/000016756.dat"),
        ("5.dat", "tests/data/000009229.dat"),
        ("6.dat", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join("foo").join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    Ok(())
}

#[test]
fn pica_split_template() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--template")
        .arg("CHUNK_{}.dat")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("CHUNK_0.dat", "tests/data/1004916019.dat"),
        ("CHUNK_1.dat", "tests/data/119232022.dat"),
        ("CHUNK_2.dat", "tests/data/000008672.dat"),
        ("CHUNK_3.dat", "tests/data/000016586.dat"),
        ("CHUNK_4.dat", "tests/data/000016756.dat"),
        ("CHUNK_5.dat", "tests/data/000009229.dat"),
        ("CHUNK_6.dat", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    // config
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[split]
template = "CHUNK_{}.dat"
"#,
        )
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("CHUNK_0.dat", "tests/data/1004916019.dat"),
        ("CHUNK_1.dat", "tests/data/119232022.dat"),
        ("CHUNK_2.dat", "tests/data/000008672.dat"),
        ("CHUNK_3.dat", "tests/data/000016586.dat"),
        ("CHUNK_4.dat", "tests/data/000016756.dat"),
        ("CHUNK_5.dat", "tests/data/000009229.dat"),
        ("CHUNK_6.dat", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        assert_eq!(
            read_to_string(outdir.join(filename)).unwrap(),
            read_to_string(sample).unwrap()
        );
    }

    Ok(())
}

#[test]
fn pica_split_gzip() -> TestResult {
    // flag
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("0.dat.gz", "tests/data/1004916019.dat"),
        ("1.dat.gz", "tests/data/119232022.dat"),
        ("2.dat.gz", "tests/data/000008672.dat"),
        ("3.dat.gz", "tests/data/000016586.dat"),
        ("4.dat.gz", "tests/data/000016756.dat"),
        ("5.dat.gz", "tests/data/000009229.dat"),
        ("6.dat.gz", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        let expected = read_to_string(sample).unwrap();

        let mut gz = GzDecoder::new(File::open(outdir.join(filename)).unwrap());
        let mut actual = String::new();
        gz.read_to_string(&mut actual).unwrap();

        assert_eq!(actual, expected);
    }

    // template
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--template")
        .arg("CHUNK_{}.dat.gz")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("CHUNK_0.dat.gz", "tests/data/1004916019.dat"),
        ("CHUNK_1.dat.gz", "tests/data/119232022.dat"),
        ("CHUNK_2.dat.gz", "tests/data/000008672.dat"),
        ("CHUNK_3.dat.gz", "tests/data/000016586.dat"),
        ("CHUNK_4.dat.gz", "tests/data/000016756.dat"),
        ("CHUNK_5.dat.gz", "tests/data/000009229.dat"),
        ("CHUNK_6.dat.gz", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        let expected = read_to_string(sample).unwrap();

        let mut gz = GzDecoder::new(File::open(outdir.join(filename)).unwrap());
        let mut actual = String::new();
        gz.read_to_string(&mut actual).unwrap();

        assert_eq!(actual, expected);
    }

    // config
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[split]
template = "CHUNK_{}.dat.gz"
"#,
        )
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    let expected = [
        ("CHUNK_0.dat.gz", "tests/data/1004916019.dat"),
        ("CHUNK_1.dat.gz", "tests/data/119232022.dat"),
        ("CHUNK_2.dat.gz", "tests/data/000008672.dat"),
        ("CHUNK_3.dat.gz", "tests/data/000016586.dat"),
        ("CHUNK_4.dat.gz", "tests/data/000016756.dat"),
        ("CHUNK_5.dat.gz", "tests/data/000009229.dat"),
        ("CHUNK_6.dat.gz", "tests/data/121169502.dat"),
    ];

    for (filename, sample) in expected {
        let expected = read_to_string(sample).unwrap();

        let mut gz = GzDecoder::new(File::open(outdir.join(filename)).unwrap());
        let mut actual = String::new();
        gz.read_to_string(&mut actual).unwrap();

        assert_eq!(actual, expected);
    }

    Ok(())
}

#[test]
fn pica_split_invalid_chunk_size() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("0")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr("error: chunk size < 1\n")
        .stdout(predicate::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("abc")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr("error: invalid chunk size\n");

    Ok(())
}

#[test]
fn pica_split_skip_invalid() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    // assert!(outdir.read_dir()?.next().is_none());

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("split")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr("Pica Error: Invalid record on line 1.\n")
        .stdout(predicate::str::is_empty());

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("split")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    // assert!(outdir.read_dir()?.next().is_none());

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[split]
skip-invalid = true
"#,
        )
        .arg("split")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    // assert!(outdir.read_dir()?.next().is_none());

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[split]
skip-invalid = true
"#,
        )
        .arg("split")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    // assert!(outdir.read_dir()?.next().is_none());

    let tempdir = Builder::new().tempdir().unwrap();
    let outdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[split]
skip-invalid = false
"#,
        )
        .arg("split")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("1")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::is_empty());

    // assert!(outdir.read_dir()?.next().is_none());

    Ok(())
}
