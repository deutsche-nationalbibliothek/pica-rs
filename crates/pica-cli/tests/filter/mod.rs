use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

mod cardinality;
mod connectives;
mod exists;
mod r#in;
mod regex;
mod regex_set;
mod relation;

#[test]
fn filter_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_stdin() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "003@.0 == '118540238'"])
        .write_stdin(read_to_string(data_dir().join("goethe.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "003@.0 == '118540238'", "-"])
        .write_stdin(read_to_string(data_dir().join("goethe.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .arg(data_dir().join("ada.dat"))
        .arg("-")
        .arg(data_dir().join("math.dat.gz"))
        .write_stdin(read_to_string(data_dir().join("goethe.dat"))?)
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_output() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("goethe.dat"))?,
        read_to_string(out.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn filter_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "003@?"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty().not())
        .stderr(predicates::str::contains(
            "parse erorr: invalid record on line 12",
        ));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@?"])
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
fn filter_invert_match() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-v", "003@.0 == '040379442'"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_ignore_case() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-i", "002@.0 == 'TP1'"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "002@.0 == 'TP1'"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_strsim_threshold() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028A.a =* 'Lovelaca'"])
        .args(["--strsim-threshold", "75"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028A.a =* 'Lovelaca'"])
        .args(["--strsim-threshold", "90"])
        .arg(data_dir().join("math.dat.gz"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_keep() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--keep", "003@, 002@"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "002@ \u{1f}0Tpz\u{1e}003@ \u{1f}0118540238\u{1e}\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_discard() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--discard", "002@,012A/*"])
        .write_stdin(
            "002@ \u{1f}0Tpz\u{1e}003@ \u{1f}0118540238\u{1e}\n",
        )
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("003@ \u{1f}0118540238\u{1e}\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_expr_file() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();

    let expr_file = temp_dir.child("expr.txt");
    expr_file.write_str("003@.0 == '118540238'").unwrap();

    let assert = cmd
        .args(["filter", "-s"])
        .args(["-F", expr_file.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn filter_allow() -> TestResult {
    // IDN
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // PPN
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn\n118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // PPN+IDN
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn,ppn\n118607626,118540238").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // empty allow list
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let allow = temp_dir.child("allow.csv");
    allow.write_str("idn\n").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn filter_deny() -> TestResult {
    // IDN
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("deny.csv");
    deny.write_str(
        "idn\n\
        118607626\n\
        040993396\n\
        04099337X\n\
        040991970\n\
        040991989\n\
        041274377\n\
        964262134\n\
        040533093\n\
        040309606\n\
        040128997\n\
        040651053\n",
    )
    .unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // PPN
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("deny.csv");
    deny.write_str(
        "idn\n\
        118607626\n\
        040993396\n\
        04099337X\n\
        040991970\n\
        040991989\n\
        041274377\n\
        964262134\n\
        040533093\n\
        040309606\n\
        040128997\n\
        040651053\n",
    )
    .unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    // empty deny list
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let deny = temp_dir.child("deny.csv");
    deny.write_str("idn\n").unwrap();

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());
    temp_dir.close().unwrap();

    Ok(())
}

#[test]
fn filter_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["--limit", "1"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .args(["filter", "-l", "2", "003@?"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("goethe.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let output = read_to_string(out.path())?;
    let mut goethe = output.lines().nth(0).unwrap().to_string();
    goethe.push('\n');
    assert_eq!(read_to_string(data_dir().join("goethe.dat"))?, goethe);

    let mut ada = output.lines().nth(1).unwrap().to_string();
    ada.push('\n');
    assert_eq!(read_to_string(data_dir().join("ada.dat"))?, ada);

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn filter_and() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--and", "002@.0 == 'Tpz'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_and_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--and", "002@.0 == 'Tpz'"])
        .args(["--not", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--not", "002@.0 == 'Tp1'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_or() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "002@.0 == 'Tuz'"])
        .args(["--or", "003@.0 == '118540238'"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn filter_gzip() -> TestResult {
    // Flag
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat.gz");

    let assert = cmd
        .args(["filter", "-s", "--gzip", "003@.0 == '118540238'"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("118540238\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();

    // Filename
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat.gz");

    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("118540238\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn filter_append() -> TestResult {
    // Flag
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .args(["filter", "-s", "003@?"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-s", "003@?", "--append"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("goethe.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["select", "003@.0"])
        .arg(out.to_str().unwrap())
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("119232022\n118540238\n"))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn filter_tee() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let tee = temp_dir.child("tee.dat");

    let assert = cmd
        .args(["filter", "-s", "003@.0 == '118540238'"])
        .args(["--tee", tee.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(
            data_dir().join("goethe.dat"),
        ))
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("goethe.dat"))?,
        read_to_string(tee.path())?
    );

    temp_dir.close().unwrap();
    Ok(())
}

/// https://github.com/deutsche-nationalbibliothek/pica-rs/issues/907
#[test]
fn filter_no_ppn() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;

    let data = "036E/00 \x1faSpringer-Lehrbuch\x1e036E/01 \x1faSpringer-Link\x1fpBücher\x1e\n";
    let assert =
        cmd.args(["filter", "....?"]).write_stdin(data).assert();
    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(data))
        .stderr(predicates::str::is_empty());
    Ok(())
}
