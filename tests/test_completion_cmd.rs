mod common;

use common::CliRunner;
use tempdir::TempDir;

#[test]
fn test_bash_completion() {
    let tempdir = TempDir::new("completion_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "completion",
        &[
            "-o",
            outdir.join("completion.bash").to_str().unwrap(),
            "bash",
        ],
    );
    assert!(result.status.success());
    assert!(outdir.join("completion.bash").exists());
}

#[test]
fn test_zsh_completion() {
    let tempdir = TempDir::new("completion_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "completion",
        &["-o", outdir.join("completion.zsh").to_str().unwrap(), "zsh"],
    );
    assert!(result.status.success());
    assert!(outdir.join("completion.zsh").exists());
}

#[test]
fn test_fish_completion() {
    let tempdir = TempDir::new("completion_test").unwrap();
    let outdir = tempdir.path();

    let result = CliRunner::new().invoke(
        "completion",
        &[
            "-o",
            outdir.join("completion.fish").to_str().unwrap(),
            "fish",
        ],
    );
    assert!(result.status.success());
    assert!(outdir.join("completion.fish").exists());
}
