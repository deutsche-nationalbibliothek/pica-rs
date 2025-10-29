use crate::prelude::*;

#[test]
fn regex_set() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "002@.0 =~ ['^T[bfg][1-3z]$', '^Tp[1z]$']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "002@.0 =~ ['^T[bfg][1-3z]$', '^Tp[23z]$']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "-i"])
        .arg("002@.0 =~ ['^t[BFG][1-3z]$', '^tP[1Z]$']")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("filter")
        .arg("042A{ ALL a =~ ['30p', '5p$'] }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("filter")
        .arg("042A{ ALL a =~ ['30p', 'p$'] }")
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
fn regex_set_inverted() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "002@.0 !~ ['^T[bfg][1-3z]$', '^Tp[1z]$']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "002@.0 !~ ['^T[bfg][1-3z]$', '^Tp[23z]$']"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::path::eq_file(data_dir().join("ada.dat")))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["filter", "-i"])
        .arg("002@.0 !~ ['^t[BFG][1-3z]$', '^tP[1Z]$']")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    Ok(())
}
