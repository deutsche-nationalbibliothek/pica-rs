#[test]
fn invalid() {
    trycmd::TestCases::new()
        .case("tests/snapshot/invalid/*.trycmd")
        .case("tests/snapshot/invalid/*.toml");
}
