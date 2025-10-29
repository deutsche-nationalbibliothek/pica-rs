use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn slice_default() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_start_end() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .args(["--start", "2", "--end", "5"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_start() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .args(["--start", "9"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_end() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .args(["--end", "3"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_length() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .args(["--length", "5"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_gzip() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat.gz");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_append() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .args(["--length", "2"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid", "--append"])
        .args(["--start", "5", "--length", "2"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("4\n"))
        .stderr(predicates::str::is_empty());

    outdir.close().unwrap();
    Ok(())
}

#[test]
fn slice_skip_invalid() -> TestResult {
    let outdir = TempDir::new().unwrap();
    let out = outdir.child("slice.dat");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice", "--skip-invalid"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["slice"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    outdir.close().unwrap();
    Ok(())
}
