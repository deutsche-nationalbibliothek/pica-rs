use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

use crate::prelude::*;

#[test]
fn completions_bash() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("pica.bash");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("completions")
        .arg("bash")
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::exists().eval(out.path()));

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn completions_stdout() -> TestResult {
    for shell in ["bash", "zsh", "elvish"] {
        let mut cmd = pica_cmd();
        let assert = cmd.arg("completions").arg(shell).assert();
        assert
            .success()
            .code(0)
            .stdout(predicates::str::is_empty().not())
            .stderr(predicates::str::is_empty());
    }

    Ok(())
}

#[test]
fn completions_zsh() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("pica.zsh");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("completions")
        .arg("zsh")
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::exists().eval(out.path()));

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn completions_elvish() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("elvish.sh");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("completions")
        .arg("elvish")
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::exists().eval(out.path()));

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn completions_fish() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("completions.fish");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("completions")
        .arg("fish")
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::exists().eval(out.path()));

    temp_dir.close().unwrap();
    Ok(())
}

#[test]
fn completions_powershell() -> TestResult {
    let temp_dir = TempDir::new().unwrap();
    let out = temp_dir.child("completions.ps1");

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("completions")
        .arg("powershell")
        .args(["-o", out.to_str().unwrap()])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    assert!(predicates::path::exists().eval(out.path()));

    temp_dir.close().unwrap();
    Ok(())
}
