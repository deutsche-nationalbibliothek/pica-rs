use crate::support::{
    CommandBuilder, MatchResult, SAMPLE1, SAMPLE2, SAMPLE3, SAMPLE4, SAMPLE5,
    SAMPLE6, SAMPLE7,
};
use flate2::read::GzDecoder;
use std::fs::{read_to_string, remove_file, File};
use std::io::Read;
use tempfile::Builder;

#[test]
fn partition_by_bbg() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .arg("002@.[01]")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string("Ts1.dat").unwrap(), SAMPLE1);
    remove_file("Ts1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string("Tp1.dat").unwrap(), exprected);
    remove_file("Tp1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string("Tb1.dat").unwrap(), exprected);
    remove_file("Tb1.dat").unwrap();

    Ok(())
}

#[test]
fn partition_template_str() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args("--template BBG_{}.dat")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string("BBG_Ts1.dat").unwrap(), SAMPLE1);
    remove_file("BBG_Ts1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string("BBG_Tp1.dat").unwrap(), exprected);
    remove_file("BBG_Tp1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string("BBG_Tb1.dat").unwrap(), exprected);
    remove_file("BBG_Tb1.dat").unwrap();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .with_config(
            r#"
[partition]
template = "BBG_{}.dat"
"#,
        )
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string("BBG_Ts1.dat").unwrap(), SAMPLE1);
    remove_file("BBG_Ts1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string("BBG_Tp1.dat").unwrap(), exprected);
    remove_file("BBG_Tp1.dat").unwrap();

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string("BBG_Tb1.dat").unwrap(), exprected);
    remove_file("BBG_Tb1.dat").unwrap();

    Ok(())
}

#[test]
fn partition_output_dir1() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string(outdir.join("Ts1.dat")).unwrap(), SAMPLE1);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), exprected);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string(outdir.join("Tb1.dat")).unwrap(), exprected);

    Ok(())
}

#[test]
fn partition_output_dir2() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    assert_eq!(read_to_string(outdir.join("Ts1.dat")).unwrap(), SAMPLE1);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE2);
    exprected.push_str(SAMPLE7);

    assert_eq!(read_to_string(outdir.join("Tp1.dat")).unwrap(), exprected);

    let mut exprected = String::new();
    exprected.push_str(SAMPLE3);
    exprected.push_str(SAMPLE4);
    exprected.push_str(SAMPLE5);
    exprected.push_str(SAMPLE6);

    assert_eq!(read_to_string(outdir.join("Tb1.dat")).unwrap(), exprected);

    Ok(())
}

#[test]
fn partition_skip_invalid() -> MatchResult {
    // CLI flag
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    // global config
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .with_config(
            r#"
[global]
skip-invalid = true
"#,
        )
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    // partition config
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .with_config(
            r#"
[partition]
skip-invalid = true
"#,
        )
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    // global/partition config
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .with_config(
            r#"
[global]
skip-invalid = false

[partition]
skip-invalid = true
"#,
        )
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    // CLI flag overwrites config
    let tempdir = Builder::new().prefix("pica-partition").tempdir().unwrap();
    let outdir = tempdir.path().join("bbg");

    CommandBuilder::new("partition")
        .with_config(
            r#"
[global]
skip-invalid = false

[partition]
skip-invalid = false
"#,
        )
        .arg("--skip-invalid")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    Ok(())
}

#[test]
fn partition_gzip_output() -> MatchResult {
    // filename extension
    let tempdir = Builder::new()
        .prefix("pica-partition-gzip")
        .tempdir()
        .unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args("--template {}.dat.gz")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat.gz")).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // gzip flag
    let tempdir = Builder::new()
        .prefix("pica-partition-gzip")
        .tempdir()
        .unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .arg("--gzip")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat.gz")).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // gzip flag #2
    let tempdir = Builder::new()
        .prefix("pica-partition-gzip")
        .tempdir()
        .unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .arg("--gzip")
        .args("--template {}.dat")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat")).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    // config
    let tempdir = Builder::new()
        .prefix("pica-partition-gzip")
        .tempdir()
        .unwrap();
    let outdir = tempdir.path();

    CommandBuilder::new("partition")
        .with_config(
            r#"
[partition]
gzip = true
"#,
        )
        .arg("--skip-invalid")
        .args("--template {}.dat")
        .args(format!("--outdir {}", outdir.to_str().unwrap()))
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .run()?;

    let mut gz = GzDecoder::new(File::open(outdir.join("Ts1.dat")).unwrap());
    let mut s = String::new();
    gz.read_to_string(&mut s).unwrap();

    assert_eq!(SAMPLE1, s);

    Ok(())
}

#[test]
fn partition_no_path() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_path1() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("00!@.0")
        .arg("tests/data/dump.dat.gz")
        .with_stderr("Pica Error: Invalid path expression\n")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_path2() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("--skip-invalid")
        .args("--outdir /root/foo")
        .arg("002@.0")
        .arg("tests/data/dump.dat.gz")
        .with_status(1)
        .run()?;

    Ok(())
}

#[test]
fn partition_invalid_file() -> MatchResult {
    CommandBuilder::new("partition")
        .arg("002@.0")
        .arg("tests/data/invalid.dat")
        .with_stderr("Pica Error: Invalid record on line 1.\n")
        .with_status(1)
        .run()?;

    Ok(())
}
