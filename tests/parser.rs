extern crate pica;

use pica::Path;
use pica::{parse_path, parse_path_list};
use std::str::FromStr;

#[test]
fn test_parse_path() {
    let data: Vec<(&str, Path)> = vec![
        ("003@.0", Path::new("003@", None, '0', None)),
        ("  003@.0 ", Path::new("003@", None, '0', None)),
        ("012A/01.0", Path::new("012A", Some("01"), '0', None)),
        ("003@.0[0]", Path::new("003@", None, '0', Some(0))),
        ("012A/00.0[1]", Path::new("012A", Some("00"), '0', Some(1))),
    ];

    for (input, expected) in data {
        assert_eq!(parse_path(input), Ok(("", expected)));
    }
}

#[test]
fn test_parse_path_list() {
    let expected = vec![
        Path::from_str("003@.0").unwrap(),
        Path::from_str("002@.0[0]").unwrap(),
    ];

    match parse_path_list(" 003@.0, 002@.0[0]  ") {
        Ok((_, actual)) => assert_eq!(expected, actual),
        _ => unreachable!(),
    }
}
