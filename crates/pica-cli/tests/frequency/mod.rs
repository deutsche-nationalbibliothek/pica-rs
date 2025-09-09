use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn frequency_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,2\nTs1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_alias() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["freq", "-s", "002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,2\nTs1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_output() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("freqs.csv");

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(read_to_string(out.path())?, "Tp1,2\nTs1,1\n");

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn frequency_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "002@.0"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_unique() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "012A.0"])
        .write_stdin(
            "003@ \x1f0118540238\x1e012A \x1f0abc\x1f0abc\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("abc,2\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "--unique", "012A.0"])
        .write_stdin(
            "003@ \x1f0118540238\x1e012A \x1f0abc\x1f0abc\x1e\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("abc,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_reverse() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-r", "002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ts1,1\nTp1,2\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_num() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "-n", "2", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tu1,6\nTsz,2\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-l", "1", "002@.0"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tpz,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_threshold() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--threshold", "2"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tu1,6\nTsz,2\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_squash() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "008A.a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("f,1\ns,1\nz,1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "--squash", "008A.a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("s|z|f,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_merge() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "008[AB].a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("f,1\nk,1\ns,1\nv,1\nw,1\nz,1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "--merge", "008[AB].a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("s|z|f|w|k|v,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_where() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--where", "002@.0 =^ 'Ts'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ts1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_where_and() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--where", "002@.0 =^ 'T'"])
        .args(["--and", "002@.0 =$ 's1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ts1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_where_or() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--where", "002@.0 =^ 'Ts'"])
        .args(["--or", "002@.0 =^ 'Tp'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,2\nTs1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_where_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--where", "002@.0 =^ 'T'"])
        .args(["--not", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ts1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_where_and_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--where", "002@.0 =^ 'T'"])
        .args(["--and", "002@.0 =$ 's1'"])
        .args(["--not", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ts1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_header() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--header", "bbg,cnt"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("bbg,cnt\nTp1,2\nTs1,1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["--header", "bbg, cnt"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("bbg,cnt\nTp1,2\nTs1,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_tsv() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "-s", "--tsv", "002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1\t2\nTs1\t1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_translit() -> TestResult {
    // no translit
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{0308}here Algebra,1\n"))
        .stderr(predicates::str::is_empty());

    // NFD
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "041@{ a | a =^ 'H'}"])
        .args(["--translit", "nfc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Höhere Algebra,1\n"))
        .stderr(predicates::str::is_empty());

    // NFKC
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "041@{ a | a =^ 'H'}"])
        .args(["--translit", "nfkc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Höhere Algebra,1\n"))
        .stderr(predicates::str::is_empty());

    // NFD
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "041@{ a | a =^ 'H'}"])
        .args(["--translit", "nfd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{0308}here Algebra,1\n"))
        .stderr(predicates::str::is_empty());

    // NFKD
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["frequency", "041@{ a | a =^ 'H'}"])
        .args(["--translit", "nfkd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{0308}here Algebra,1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn frequency_allow() -> TestResult {
    // IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn\n118540238\n040991970\n040991989\n")?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tu1,2\nTpz,1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("allow.csv");
    allow.write_str("ppn\n118540238\n040991970\n040991989\n")?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tu1,2\nTpz,1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN+IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("allow.csv");
    allow.write_str(
        "ppn,idn\n118540238,118607626\n040991970,040993396\n040991989,\n",)?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tu1,2\nTpz,1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn frequency_deny() -> TestResult {
    // PPN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("deny.csv");
    allow.write_str("ppn\n040011569\n")?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-D", allow.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,1\n"))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("deny.csv");
    allow.write_str("idn\n040011569\n")?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-D", allow.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // IDN+PPN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pica")?;
    let allow = temp_dir.child("deny.csv");
    allow.write_str("idn,119232022\nppn\n040011569\n")?;

    let assert = cmd
        .args(["frequency", "-s", "002@.0"])
        .args(["-D", allow.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1,1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}
