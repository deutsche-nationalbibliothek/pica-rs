#[test]
fn cat() {
    trycmd::TestCases::new()
        .case("tests/snapshot/cat/*.toml")
        .case("tests/snapshot/cat/*.trycmd");
}

#[test]
fn completions() {
    trycmd::TestCases::new()
        .case("tests/snapshot/completions/*.toml")
        .case("tests/snapshot/completions/*.trycmd");
}

#[test]
fn count() {
    trycmd::TestCases::new()
        .case("tests/snapshot/count/*.toml")
        .case("tests/snapshot/count/*.trycmd");
}

#[test]
fn filter() {
    trycmd::TestCases::new()
        .case("tests/snapshot/filter/*.toml")
        .case("tests/snapshot/filter/*.trycmd");
}

#[test]
fn frequency() {
    trycmd::TestCases::new()
        .case("tests/snapshot/frequency/*.toml")
        .case("tests/snapshot/frequency/*.trycmd");
}

#[test]
fn invalid() {
    trycmd::TestCases::new()
        .case("tests/snapshot/invalid/*.toml")
        .case("tests/snapshot/invalid/*.trycmd");
}

#[test]
fn partition() {
    trycmd::TestCases::new()
        .case("tests/snapshot/partition/*.toml")
        .case("tests/snapshot/partition/*.trycmd");
}

#[test]
fn select() {
    trycmd::TestCases::new()
        .case("tests/snapshot/select/*.toml")
        .case("tests/snapshot/select/*.trycmd");
}

#[test]
fn slice() {
    trycmd::TestCases::new()
        .case("tests/snapshot/slice/*.toml")
        .case("tests/snapshot/slice/*.trycmd");
}

#[test]
fn split() {
    trycmd::TestCases::new()
        .case("tests/snapshot/split/*.toml")
        .case("tests/snapshot/split/*.trycmd");
}
