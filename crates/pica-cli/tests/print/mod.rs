use std::fs::read_to_string;

use assert_fs::TempDir;
use assert_fs::prelude::*;
use unicode_normalization::UnicodeNormalization;

use crate::prelude::*;

#[test]
fn print_stdout() -> TestResult {
    let mut cmd = pica_cmd();
    let assert =
        cmd.arg("print").arg(data_dir().join("ada.dat")).assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_output() -> TestResult {
    let mut cmd = pica_cmd();
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.txt");

    let assert = cmd
        .arg("print")
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let actual = read_to_string(out.path())?;
    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert_eq!(expected, actual);

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn print_limit() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("out.txt");

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-l", "1"])
        .args(["-o", out.to_str().unwrap()])
        .arg(data_dir().join("ada.dat"))
        .arg(data_dir().join("algebra.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let actual = read_to_string(out.path())?;
    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert_eq!(expected, actual);

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn print_where() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232022\""])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_where_and() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232022\""])
        .args(["--and", "002@.0 == \"Tp1\""])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_where_and_not() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232022\""])
        .args(["--and", "002@.0 == \"Tp1\""])
        .args(["--not", "002@.0 == \"Tpz\""])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_where_or() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232023\""])
        .args(["--or", "003@.0 == \"119232022\""])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_where_not() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232022\""])
        .args(["--not", "002@.0 == \"Tpz\""])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--where", "003@.0 == \"119232022\""])
        .args(["--not", "002@.0 == \"Tp1\""])
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
fn print_allow() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn\n119232022\n")?;

    let assert = cmd
        .args(["print", "-A", allow.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    // filter set column
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("xyz\n119232022\n")?;

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "xyz"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    // filter set source
    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("gnd_id\n119232022\n")?;

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "gnd_id"])
        .args(["--filter-set-source", "007K{ 0 | a == 'gnd' }"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_deny() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = pica_cmd();

    let deny = temp_dir.child("DENY.csv");
    deny.write_str("ppn\n040011569\n")?;

    let assert = cmd
        .args(["print", "-D", deny.to_str().unwrap()])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    // filter set column
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("xyz\n040011569\n")?;

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "xyz"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    // filter set source
    let deny = temp_dir.child("DENY.csv");
    deny.write_str("gnd_id\n4001156-2\n")?;

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-D", deny.to_str().unwrap()])
        .args(["--filter-set-column", "gnd_id"])
        .args(["--filter-set-source", "007K{ 0 | a == 'gnd' }"])
        .arg(data_dir().join("algebra.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfc() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("algebra.txt"))?
        .chars()
        .nfc()
        .collect::<String>();

    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfkc() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfkc"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("algebra.txt"))?
        .chars()
        .nfkc()
        .collect::<String>();

    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfd() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("algebra.txt"))?
        .chars()
        .nfd()
        .collect::<String>();

    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_translit_nfkd() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("print")
        .args(["--translit", "nfkd"])
        .arg(data_dir().join("algebra.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("algebra.txt"))?
        .chars()
        .nfkd()
        .collect::<String>();

    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn print_skip_invalid() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print", "-s"])
        .arg(data_dir().join("invalid.dat"))
        .arg(data_dir().join("ada.dat"))
        .assert();

    let mut expected = read_to_string(data_dir().join("ada.txt"))?;
    if cfg!(windows) {
        expected = expected.replace('\r', "");
    }

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(expected))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["print"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    Ok(())
}
