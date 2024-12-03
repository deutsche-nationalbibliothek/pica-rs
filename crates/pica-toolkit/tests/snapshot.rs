#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/snapshot/explode/*.toml")
        .case("tests/snapshot/explode/*.trycmd")
        .case("tests/snapshot/filter/*.toml")
        .case("tests/snapshot/filter/*.trycmd")
        .case("tests/snapshot/select/*.toml")
        .case("tests/snapshot/select/*.trycmd");
}
