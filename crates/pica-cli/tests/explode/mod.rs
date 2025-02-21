use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use super::prelude::*;

#[test]
fn explode_stdout() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "main"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let output = read_to_string(data_dir().join("ada.dat"))?;
    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_local() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "local"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_copy() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_output() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat");

    let assert = cmd
        .args(["explode", "main"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert_eq!(
        read_to_string(data_dir().join("ada.dat"))?,
        read_to_string(out.path())?
    );

    Ok(())
}

#[test]
fn explode_gzip() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.dat.gz");

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "main"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("concat").arg(out.as_os_str()).assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let out = temp_dir.child("out.dat.gz");
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "--gzip", "main"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd.arg("concat").arg(out.as_os_str()).assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn explode_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--limit", "2"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--limit", "1"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--limit", "0"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "-s", "main"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "main"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse erorr: invalid record on line 1",
        ));

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "-s", "main"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let output = read_to_string(data_dir().join("ada.dat"))?;
    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_where() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--where", "101@.a == '1'"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_where_and() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--where", "101@.a == '1'"])
        .args(["--and", "002@.0 == 'Abvz'"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_where_and_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--where", "101@.a == '1'"])
        .args(["--and", "002@.0 == 'Abvz'"])
        .args(["--not", "203@/*.0 == '1234567890'"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_where_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--where", "101@.a == '1'"])
        .args(["--not", "203@/*.0 =$ '0'"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_where_or() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "copy"])
        .args(["--where", "203@/*.0 == '0123456789'"])
        .args(["--or", "203@/*.0 == '345678901'"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  002@ \u{1f}0Abvz\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_keep() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "local"])
        .args(["--keep", "003@,101@,203@/*"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn explode_discard() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["explode", "local"])
        .args(["--discard", "002@"])
        .arg(data_dir().join("COPY.dat.gz"))
        .assert();

    let output = "003@ \u{1f}0123456789\u{1e}\
                  101@ \u{1f}a1\u{1e}\
                  203@/01 \u{1f}00123456789\u{1e}\
                  203@/02 \u{1f}01234567890\u{1e}\n\
                  003@ \u{1f}0123456789\u{1e}\
                  101@ \u{1f}a2\u{1e}\
                  203@/01 \u{1f}0345678901\u{1e}\n";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(output))
        .stderr(predicates::str::is_empty());

    Ok(())
}
