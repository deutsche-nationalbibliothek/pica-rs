use std::fs::read_to_string;

use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

#[test]
fn count_write_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "records: 12\nfields: 1035\nsubfields: 3973\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_output() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("counts.csv");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "records: 12\nfields: 1035\nsubfields: 3973\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_write_append() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("counts.csv");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--csv"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--csv", "--append", "--no-header"])
        .args(["--where", "002@.0 =^ 'Ts'"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        "records,fields,subfields\n2,484,1514\n3,105,295\n",
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_write_csv() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--csv"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "records,fields,subfields\n12,1035,3973\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_tsv() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--tsv"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "records\tfields\tsubfields\n12\t1035\t3973\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_records() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_fields() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--fields"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1035\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_subfields() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--subfields"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3973\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_write_no_header() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--csv", "--no-header"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12,1035,3973\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--tsv", "--no-header"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("12\t1035\t3973\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_where() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_where_and() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["--where", "002@.0 =^ 'T'"])
        .args(["--and", "002@.0 =$ 'z'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("3\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_where_not() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--not", "002@.0 =$ 'z'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("1\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_where_or() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--or", "002@.0 =^ 'Ts'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("5\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_allow() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str("ppn\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_deny() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let deny = temp_dir.child("allow.csv");
    deny.write_str("ppn\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("10\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_filter_set_column() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str("id\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "id"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_filter_set_source() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("allow.csv");
    allow.write_str("bbg\nTpz\nTp1\n")?;

    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-source", "002@.0"])
        .args(["--filter-set-column", "bbg"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn count_ignore_case() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .args(["-i", "--where", "002@.0 =^ 'tp'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("2\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn count_skip_invalid() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "-s", "--records"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("0\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["count", "--records"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::starts_with(
            "error: parse error: invalid record on line 1",
        ));

    Ok(())
}
