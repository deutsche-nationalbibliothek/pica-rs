#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;
use pica_matcher::RecordMatcher;

fuzz_target!(|data: &[u8]| {
    let _result = RecordMatcher::try_from(data);
});
