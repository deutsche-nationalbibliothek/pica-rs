#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/snapshot/cat/*.toml")
        .case("tests/snapshot/cat/*.trycmd")
        .case("tests/snapshot/completions/*.toml")
        .case("tests/snapshot/completions/*.trycmd")
        .case("tests/snapshot/count/*.toml")
        .case("tests/snapshot/count/*.trycmd")
        .case("tests/snapshot/explode/*.toml")
        .case("tests/snapshot/explode/*.trycmd")
        .case("tests/snapshot/filter/*.toml")
        .case("tests/snapshot/filter/*.trycmd")
        .case("tests/snapshot/frequency/*.toml")
        .case("tests/snapshot/frequency/*.trycmd")
        .case("tests/snapshot/hash/*.toml")
        .case("tests/snapshot/hash/*.trycmd")
        .case("tests/snapshot/invalid/*.toml")
        .case("tests/snapshot/invalid/*.trycmd")
        .case("tests/snapshot/partition/*.toml")
        .case("tests/snapshot/partition/*.trycmd")
        .case("tests/snapshot/print/*.toml")
        .case("tests/snapshot/print/*.trycmd")
        .case("tests/snapshot/sample/*.toml")
        .case("tests/snapshot/sample/*.trycmd")
        .case("tests/snapshot/select/*.toml")
        .case("tests/snapshot/select/*.trycmd")
        .case("tests/snapshot/slice/*.toml")
        .case("tests/snapshot/slice/*.trycmd")
        .case("tests/snapshot/split/*.toml")
        .case("tests/snapshot/split/*.trycmd");
}

#[test]
fn doc_tests() {
    trycmd::TestCases::new()
        .case("../../docs/book/src/referenz/kommandos/count.md")
        .case("../../docs/book/src/referenz/kommandos/filter.md")
        .case("../../docs/book/src/referenz/kommandos/frequency.md")
        .case("../../docs/book/src/referenz/kommandos/explode.md")
        .case("../../docs/book/src/referenz/kommandos/hash.md")
        .case("../../docs/book/src/referenz/kommandos/invalid.md")
        .case("../../docs/book/src/referenz/kommandos/partition.md")
        .case("../../docs/book/src/referenz/kommandos/print.md")
        .case("../../docs/book/src/referenz/kommandos/slice.md")
        .case("../../docs/book/src/referenz/kommandos/split.md");
}