#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/snapshot/explode/*.toml")
        .case("tests/snapshot/explode/*.trycmd")
        .case("tests/snapshot/filter/*.toml")
        .case("tests/snapshot/filter/*.trycmd")
        .case("tests/snapshot/frequency/*.toml")
        .case("tests/snapshot/frequency/*.trycmd")
        .case("tests/snapshot/print/*.toml")
        .case("tests/snapshot/print/*.trycmd")
        .case("tests/snapshot/sample/*.toml")
        .case("tests/snapshot/sample/*.trycmd")
        .case("tests/snapshot/select/*.toml")
        .case("tests/snapshot/select/*.trycmd")
        .case("tests/snapshot/slice/*.toml")
        .case("tests/snapshot/slice/*.trycmd");
}
