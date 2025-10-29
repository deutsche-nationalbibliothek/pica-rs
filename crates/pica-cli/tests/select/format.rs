use crate::prelude::*;

#[test]
fn format_string_simple() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{ a <$> (', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"Lovelace, Ada King (of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_string_with_predicate() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028R{a <$> (', ' d <*> ' (' v ')') | v in ['Vater', 'Mutter']}")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"Byron, George Gordon Byron (Vater)\"\n\
            119232022,\"Byron, Anne Isabella Milbanke Byron (Mutter)\"\n"
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_string_uppercase() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{?u a <$> (', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"LOVELACE, Ada King (of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{?u a <$> (?u ', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"LOVELACE, ADA KING (OF)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_string_lowercase() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{?l a <$> (', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"lovelace, Ada King (of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{?l a <$> (?l ', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"lovelace, ada king (of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_string_strip_whitespaces() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{ a <$> (?w ', ' d <*> ' (' c ')' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"Lovelace,AdaKing(of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_string_trim() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .arg("select")
        .arg("003@.0,028A{a <$> (?t ', ' d <*> ' (' c ') ' ) }")
        .arg(data_dir().join("ada.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(
            "119232022,\"Lovelace, Ada King (of)\"\n",
        ))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn format_strip_overread_char() -> TestResult {
    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "021A{?o a }"])
        .write_stdin(b"021A \x1fa@abc\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("abc\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = pica_cmd();
    let assert = cmd
        .args(["select", "021A{ a }"])
        .write_stdin(b"021A \x1fa@abc\x1e\n")
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("@abc\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}
