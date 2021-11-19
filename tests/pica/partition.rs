use crate::common::{CommandExt, TestContext, TestResult};
use assert_cmd::Command;
use flate2::read::GzDecoder;
use predicates::prelude::*;
use std::fs::{read_to_string, remove_file, File};
use std::io::Read;
use tempfile::Builder;

#[test]
fn pica_partition_by_bbg() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let actual = read_to_string("Tb1.dat").unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);
    remove_file("Tb1.dat").unwrap();

    // Tp1
    let actual = read_to_string("Tp1.dat").unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);
    remove_file("Tp1.dat").unwrap();

    // Ts1
    let actual = read_to_string("Ts1.dat").unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);
    remove_file("Ts1.dat").unwrap();

    Ok(())
}

#[test]
fn pica_partition_filename_template() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir)
        .arg("--template")
        .arg("BBG_{}.dat")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let actual = read_to_string(tempdir.join("BBG_Tb1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let actual = read_to_string(tempdir.join("BBG_Tp1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let actual = read_to_string(tempdir.join("BBG_Ts1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_filename_template_config() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[partition]
template = "bbg_{}.dat"
"#,
        )
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let actual = read_to_string(tempdir.join("bbg_Tb1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let actual = read_to_string(tempdir.join("bbg_Tp1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let actual = read_to_string(tempdir.join("bbg_Ts1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_output_dir1() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let actual = read_to_string(tempdir.join("Tb1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let actual = read_to_string(tempdir.join("Tp1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let actual = read_to_string(tempdir.join("Ts1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_output_dir2() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir.join("dir2"))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let actual = read_to_string(tempdir.join("dir2/Tb1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let actual = read_to_string(tempdir.join("dir2/Tp1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let actual = read_to_string(tempdir.join("dir2/Ts1.dat")).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_skip_invalid() -> TestResult {
    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success();
    assert!(tempdir.read_dir()?.next().is_none());

    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr(predicate::eq("Pica Error: Invalid record on line 1.\n"));

    let tempdir = Builder::new().tempdir().unwrap();
    let tempdir = tempdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = true
"#,
        )
        .arg("partition")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success();
    assert!(tempdir.read_dir()?.next().is_none());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[partition]
skip-invalid = true
"#,
        )
        .arg("partition")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success();
    assert!(tempdir.read_dir()?.next().is_none());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[partition]
skip-invalid = true
"#,
        )
        .arg("partition")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success();
    assert!(tempdir.read_dir()?.next().is_none());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[global]
skip-invalid = false

[partition]
skip-invalid = false
"#,
        )
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(tempdir)
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .assert();

    assert.success();
    assert!(tempdir.read_dir()?.next().is_none());

    Ok(())
}

#[test]
fn pica_partition_write_gzip_template() -> TestResult {
    let outdir = Builder::new().tempdir().unwrap();
    let outdir = outdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--template")
        .arg("{}.dat.gz")
        .arg("--outdir")
        .arg(outdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tb1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tp1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_write_gzip_flag1() -> TestResult {
    let outdir = Builder::new().tempdir().unwrap();
    let outdir = outdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("--outdir")
        .arg(outdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tb1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tp1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_write_gzip_flag2() -> TestResult {
    let outdir = Builder::new().tempdir().unwrap();
    let outdir = outdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--gzip")
        .arg("--template")
        .arg("{}.dat")
        .arg("--outdir")
        .arg(outdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tb1.dat")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tp1.dat")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_write_gzip_config() -> TestResult {
    let outdir = Builder::new().tempdir().unwrap();
    let outdir = outdir.path();

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .with_config(
            &TestContext::new(),
            r#"[partition]
gzip = true
"#,
        )
        .arg("partition")
        .arg("--skip-invalid")
        .arg("--outdir")
        .arg(outdir)
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .assert();
    assert.success();

    // Tb1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tb1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/000008672.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016586.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000016756.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/000009229.dat").unwrap());

    assert_eq!(expected, actual);

    // Tp1
    let mut gz = GzDecoder::new(File::open(outdir.join("Tp1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/119232022.dat").unwrap());
    expected.push_str(&read_to_string("tests/data/121169502.dat").unwrap());

    assert_eq!(expected, actual);

    // Ts1
    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat.gz")).unwrap());
    let mut actual = String::new();
    gz.read_to_string(&mut actual).unwrap();

    let mut expected = String::new();
    expected.push_str(&read_to_string("tests/data/1004916019.dat").unwrap());

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn pica_partition_invalid_path() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("partition")
        .arg("--skip-invalid")
        .arg("002@.!")
        .arg("tests/data/dump.dat.gz")
        .assert();

    assert
        .failure()
        .code(1)
        .stderr("Pica Error: Invalid path expression\n")
        .stdout(predicate::str::is_empty());

    Ok(())
}
