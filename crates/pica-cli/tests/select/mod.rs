use std::fs::read_to_string;

use assert_fs::TempDir;
use assert_fs::prelude::*;

use super::prelude::*;

mod format;

#[test]
fn select_csv_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040011569,Ts1\n119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_csv_output() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.csv");

    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(out.path())?,
        "040011569,Ts1\n119232022,Tp1\n"
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn select_tsv_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--tsv", "003@.0,002@.0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040011569\tTs1\n119232022\tTp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_tsv_output() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.tsv");

    let assert = cmd
        .args(["select", "--tsv", "003@.0,002@.0"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(out.path())?,
        "040011569\tTs1\n119232022\tTp1\n"
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn select_stdin() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,012B.0"])
        .write_stdin("003@ \x1f0123\x1e012B \x1f0bar\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("123,bar\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "-s", "003@.0,012B.0"])
        .args([data_dir().join("invalid.dat"), "-".into()])
        .write_stdin("003@ \x1f0123\x1e012B \x1f0bar\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("123,bar\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_skip_invalid() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "002@.0"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    let mut cmd = pica_cmd();
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
fn select_squash() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--squash", "003@.0,008A.a"])
        .arg(data_dir().join("math.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040379442,s|g\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,008A.a"])
        .args(["--squash", "--separator", "+++"])
        .arg(data_dir().join("math.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040379442,s+++g\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,008A.a"])
        .args(["--squash", "--separator", "s"])
        .arg(data_dir().join("math.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040379442,ssg\n"))
        .stderr(predicates::str::starts_with(
            "WARNING: A subfield value contains squash separator",
        ));

    Ok(())
}

#[test]
fn select_merge() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--merge", "003@.0,008A.a,008B.a"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "040011569,s,w|z|o\n119232022,s|z|f,w|k|v\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,008A.a,008B.a"])
        .args(["--merge", "--separator", "+++"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "040011569,s,w+++z+++o\n119232022,s+++z+++f,w+++k+++v\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,008A.a,008B.a"])
        .args(["--merge", "--separator", ""])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "040011569,s,wzo\n119232022,szf,wkv\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_no_empty_columns() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--no-empty-columns", "003@.0,028A.a"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Lovelace\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_unique() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--unique", "004B.a"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("saz\npik\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_ignore_case() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--ignore-case"])
        .args(["003@.0,041[A@]{ a | a == 'algebra'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040011569,Algebra\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_strsim_threshold() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,041[AP]{ a | a =* 'Algebra'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "040011569,Algebra\n040011569,Algebra\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--strsim-threshold", "60"])
        .arg("041[AP]{ a | a =* 'Algebra'}")
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "Algebra\nAlgebra\nAlge\u{0300}bre\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_limit() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--limit", "2", "003@.0,002@.0"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040379442,Tsz\n040011569,Ts1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "-l", "1", "003@.0,002@.0"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040379442,Tsz\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--limit", "0", "003@.0,002@.0"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "040379442,Tsz\n040011569,Ts1\n119232022,Tp1\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_translit() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{308}here Algebra\n"))
        .stderr(predicates::str::is_empty());

    // NFD
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--translit", "nfd", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{308}here Algebra\n"))
        .stderr(predicates::str::is_empty());

    // NFKD
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--translit", "nfkd", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Ho\u{308}here Algebra\n"))
        .stderr(predicates::str::is_empty());

    // NFC
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--translit", "nfc", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Höhere Algebra\n"))
        .stderr(predicates::str::is_empty());

    // NFKC
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--translit", "nfkc", "041@{ a | a =^ 'H'}"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Höhere Algebra\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_where() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["--where", "003@.0 != '040011569'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_where_and() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["--where", "003@.0 != '040011569'"])
        .args(["--and", "002@.0 =^ 'Tp'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_where_not() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["--where", "003@.0 != '040011569'"])
        .args(["--not", "002@.0 == 'Ts1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_where_and_not() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["--where", "003@.0 != '040011569'"])
        .args(["--and", "002@.0 =^ 'Tp'"])
        .args(["--not", "002@.0 == 'Ts1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_where_or() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .args(["--where", "002@.0 == 'Ts1'"])
        .args(["--or", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040011569,Ts1\n119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_allow() -> TestResult {
    // IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn\n118540238\n040991970\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tpz\nTu1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str("ppn\n118540238\n040991970\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tpz\nTu1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN+IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str(
        "idn,ppn\n040991970,118540238\n118540238,040991970\n",
    )?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tpz\nTu1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    // FILTER SET COLUMN
    let allow = temp_dir.child("allow.csv");
    allow.write_str("id\n118540238\n040991970\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "id"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tpz\nTu1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn select_deny() -> TestResult {
    // IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let deny = temp_dir.child("deny.csv");
    deny.write_str("idn\n040011569\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let deny = temp_dir.child("deny.csv");
    deny.write_str("ppn\n040011569\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // PPN+IDN
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let deny = temp_dir.child("deny.csv");
    deny.write_str("idn,ppn\n119232022,040011569\n")?;

    let assert = cmd
        .args(["select", "-s", "002@.0"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("Tp1\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn select_query_const() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0, 'abc'"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,abc\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn select_query_path() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,002@.0"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@.0,028@{ (d, a) | 4 == 'nafr'}"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Ada Augusta,Byron\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--merge", "003@.0,008[AB].a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,s|z|f|w|k|v\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "--squash", "003@.0, 008[AB].a"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,s|z|f\n119232022,w|k|v\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
#[cfg(feature = "compat")]
fn select_compat() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@ $0,002@$0"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("040011569,Ts1\n119232022,Tp1\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@$0,028@{ $d, $a | $4 == 'nafr'}"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Ada Augusta,Byron\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "003@$0,028@{ ($d, $a) | $4 == 'nafr'}"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022,Ada Augusta,Byron\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}
