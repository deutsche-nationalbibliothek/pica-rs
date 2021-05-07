use crate::support::{CommandBuilder, MatchResult};
use tempfile::Builder;

#[test]
fn test_bash_completion() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-completion").tempdir().unwrap();
    let filename = tempdir.path().join("pica.sh");

    CommandBuilder::new("completion")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("bash")
        .with_stdout_empty()
        .run()?;

    assert!(filename.exists());
    Ok(())
}

#[test]
fn test_fish_completion() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-completion").tempdir().unwrap();
    let filename = tempdir.path().join("pica.fish");

    CommandBuilder::new("completion")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("fish")
        .with_stdout_empty()
        .run()?;

    assert!(filename.exists());
    Ok(())
}

#[test]
fn test_zsh_completion() -> MatchResult {
    let tempdir = Builder::new().prefix("pica-completion").tempdir().unwrap();
    let filename = tempdir.path().join("pica.zsh");

    CommandBuilder::new("completion")
        .args(format!("--output {}", filename.to_str().unwrap()))
        .arg("zsh")
        .with_stdout_empty()
        .run()?;

    assert!(filename.exists());
    Ok(())
}
