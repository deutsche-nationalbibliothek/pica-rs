use assert_cmd::Command;

use crate::prelude::*;

#[test]
fn contains_set() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@.d =? ['September', 'August']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@.d =? ['Oktober', 'November']"])
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
fn contains_set_case_ignore() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-i", "028@.d =? ['september', 'august']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "-i", "028@.d =? ['oktober', 'november']"])
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
fn contains_set_quantifier() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@{ ANY d =? ['eptemb', 'gust'] }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@{ ANY d =? ['tobe', 'ovemb'] }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@{ ALL [ad] =? ['b', 'a'] }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["filter", "028@{ ALL [ad] =? ['x', 'y'] }"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}
