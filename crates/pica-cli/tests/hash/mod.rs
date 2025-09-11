use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

use crate::prelude::*;

const HASHES: &'static str = "\
118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n\
040993396,0361c33e1f7a80e21eecde747b2721b7884e003ac4deb8c271684ec0dc4d059a\n\
04099337X,9ea44b7968a3668f12796b33f39abf45abbdd253eede0b5232049c3dcc5b3049\n\
040991970,85998d025a6076a57441c56a4b8a64877b0a7c47fb4d93825270942704716b70\n\
040991989,564b08a820bcbc4457643969db4ad99dd1701e7185f4cd4df6b8928a2bbb2360\n\
041274377,0a578e4da5e5a6cc49776975d4d4ee877fe8e3a667be0e12f87dc47a7aa91a85\n\
964262134,78d7d3b83e5bdec16ff79de2b431c94a53ef8abfaa3db2f48ba95fde39728594\n\
040533093,ab4d24140c1d2440b00d93fca735db0a3f17a6734eefdaad559c2ed06a8c0d21\n\
040309606,f8876d13cd12cc587451b138d196956f3221a366c8960114a32b2ab52b5fd5ef\n\
040128997,aee29a116fc8a6b6c85530682ef7ce254f43f45f06e85a9275bfac8b25f2eaf4\n\
040651053,036503450bbe80fc1b08d845b7106c18e78b282e6a834bb88675fcf25aa4fd56\n";

#[test]
fn hash_single_record() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("hash").arg(data_dir().join("ada.dat")).assert();

    let row = "119232022,\
       0ff4c124bb89f27fb73549840654a0c0b6cb7b4d66dc6a20a57a4d49895d62ca";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(format!("ppn,hash\n{row}\n")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_dump() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(format!("ppn,hash\n{HASHES}")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_tsv() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "--tsv"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let row = "119232022\t\
       0ff4c124bb89f27fb73549840654a0c0b6cb7b4d66dc6a20a57a4d49895d62ca";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(format!("ppn\thash\n{row}\n")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_skip_invalid() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("invalid.dat"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n"))
        .stderr(predicates::str::is_empty());

    let mut cmd = Command::cargo_bin("pica")?;
    let assert =
        cmd.arg("hash").arg(data_dir().join("invalid.dat")).assert();

    assert
        .failure()
        .code(2)
        .stdout(predicates::ord::eq("ppn,hash\n"))
        .stderr(predicates::str::contains(
            "parse error: invalid record on line 1",
        ));

    Ok(())
}

#[test]
fn hash_header() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .arg("hash")
        .args(["--header", "idn,sha256"])
        .arg(data_dir().join("ada.dat"))
        .assert();

    let row = "119232022,\
       0ff4c124bb89f27fb73549840654a0c0b6cb7b4d66dc6a20a57a4d49895d62ca";

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq(format!("idn,sha256\n{row}\n")))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_where() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "003@.0 == '040991970'"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n040991970,85998d025a6076a57441c56a4b8a64877b0a7c47fb4d93825270942704716b70\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_where_and() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "003@.0 == '040991970'"])
        .args(["--and", "002@.0 == 'Tu1'"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n040991970,85998d025a6076a57441c56a4b8a64877b0a7c47fb4d93825270942704716b70\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_where_or() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "003@.0 == '118540238'"])
        .args(["--or", "003@.0 == '118607626'"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
            118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_where_not() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .args(["--where", "002@.0 =^ 'Tp'"])
        .args(["--not", "003@.0 == '118607626'"])
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_limit() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let assert = cmd
        .args(["hash", "-s", "-l", "2"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
            118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_allow() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("ppn\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["hash", "-s"])
        .args(["-A", allow.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
            118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_deny() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();

    let deny = temp_dir.child("DENY.csv");
    deny.write_str("ppn\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["hash", "-s"])
        .args(["-D", deny.to_str().unwrap()])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            040993396,0361c33e1f7a80e21eecde747b2721b7884e003ac4deb8c271684ec0dc4d059a\n\
            04099337X,9ea44b7968a3668f12796b33f39abf45abbdd253eede0b5232049c3dcc5b3049\n\
            040991970,85998d025a6076a57441c56a4b8a64877b0a7c47fb4d93825270942704716b70\n\
            040991989,564b08a820bcbc4457643969db4ad99dd1701e7185f4cd4df6b8928a2bbb2360\n\
            041274377,0a578e4da5e5a6cc49776975d4d4ee877fe8e3a667be0e12f87dc47a7aa91a85\n\
            964262134,78d7d3b83e5bdec16ff79de2b431c94a53ef8abfaa3db2f48ba95fde39728594\n\
            040533093,ab4d24140c1d2440b00d93fca735db0a3f17a6734eefdaad559c2ed06a8c0d21\n\
            040309606,f8876d13cd12cc587451b138d196956f3221a366c8960114a32b2ab52b5fd5ef\n\
            040128997,aee29a116fc8a6b6c85530682ef7ce254f43f45f06e85a9275bfac8b25f2eaf4\n\
            040651053,036503450bbe80fc1b08d845b7106c18e78b282e6a834bb88675fcf25aa4fd56\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_filter_set_column() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("id\n118540238\n118607626\n")?;

    let assert = cmd
        .args(["hash", "-s"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-column", "id"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
            118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}

#[test]
fn hash_filter_set_source() -> TestResult {
    let mut cmd = Command::cargo_bin("pica")?;
    let temp_dir = TempDir::new().unwrap();

    let allow = temp_dir.child("ALLOW.csv");
    allow.write_str("bbg\nTpz\nTp1\n")?;

    let assert = cmd
        .args(["hash", "-s"])
        .args(["-A", allow.to_str().unwrap()])
        .args(["--filter-set-source", "002@.0"])
        .args(["--filter-set-column", "bbg"])
        .arg(data_dir().join("DUMP.dat.gz"))
        .assert();

    assert
        .success()
        .code(0)
        .stdout(predicates::ord::eq("ppn,hash\n\
            118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3\n\
            118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee\n"))
        .stderr(predicates::str::is_empty());

    Ok(())
}
