extern crate pica;

use pica::{Filter, ParseFilterError};

#[test]
fn test_filter_decode() {
    let result = Filter::decode("003@.0 == '123456789X'");
    assert!(result.is_ok());

    let result = Filter::decode("003@.! == '123456789X'");
    assert_eq!(result, Err(ParseFilterError));
}
