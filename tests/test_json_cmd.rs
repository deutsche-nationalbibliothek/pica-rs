// mod common;
// use common::CliRunner;

// static SAMPLE1_JSON: &'static str =
// "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\"
// :\"0\",\"value\":\"123456789X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"
// subfields\":[{\"name\":\"0\",\"value\":\"Tp1\"}]},{\"name\":\"012A\",\"
// occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\"
// :\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

// static SAMPLE2_JSON: &'static str =
// "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\"
// :\"0\",\"value\":\"234567891X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"
// subfields\":[{\"name\":\"0\",\"value\":\"Tp2\"}]},{\"name\":\"012A\",\"
// occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\"
// :\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

// static SAMPLE3_JSON: &'static str =
// "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\"
// :\"0\",\"value\":\"345678912X\"}]},{\"name\":\"002@\",\"occurrence\":null,\"
// subfields\":[{\"name\":\"0\",\"value\":\"Tp1\"}]},{\"name\":\"012A\",\"
// occurrence\":\"00\",\"subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\"
// :\"a\",\"value\":\"2\"},{\"name\":\"b\",\"value\":\"1\"}]}]}";

// static SAMPLE4_JSON: &'static str =
// "{\"fields\":[{\"name\":\"003@\",\"occurrence\":null,\"subfields\":[{\"name\"
// :\"0\",\"value\":\"33445566X\"}]},{\"name\":\"012A\",\"occurrence\":null,\"
// subfields\":[{\"name\":\"a\",\"value\":\"1\"},{\"name\":\"b\",\"value\":\"1\"
// }]},{\"name\":\"012A\",\"occurrence\":null,\"subfields\":[{\"name\":\"a\",\"
// value\":\"2\"},{\"name\":\"a\",\"value\":\"3\"}]}]}";

// #[test]
// fn test_json_cmd() {
//     let result = CliRunner::new()
//         .invoke("json", &["--skip-invalid", "tests/data/empty.dat"]);
//     assert!(result.status.success());

//     assert_eq!(String::from_utf8(result.stdout).unwrap(), "[]");

//     let result = CliRunner::new()
//         .invoke("json", &["--skip-invalid", "tests/data/1.dat"]);
//     assert!(result.status.success());

//     assert_eq!(
//         String::from_utf8(result.stdout).unwrap(),
//         format!("[{}]", SAMPLE1_JSON)
//     );

//     let result = CliRunner::new()
//         .invoke("json", &["--skip-invalid", "tests/data/all.dat.gz"]);
//     assert!(result.status.success());

//     assert_eq!(
//         String::from_utf8(result.stdout).unwrap(),
//         format!(
//             "[{},{},{},{}]",
//             SAMPLE1_JSON, SAMPLE2_JSON, SAMPLE3_JSON, SAMPLE4_JSON
//         )
//     );
// }

// #[test]
// fn test_skip_invalid() {
//     let result = CliRunner::new().invoke("json",
// &["tests/data/invalid.dat"]);     assert!(!result.status.success());
// }
