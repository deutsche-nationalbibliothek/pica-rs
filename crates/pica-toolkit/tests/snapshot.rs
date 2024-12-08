#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/snapshot/filter/*.toml")
        .case("tests/snapshot/filter/*.trycmd");
}
