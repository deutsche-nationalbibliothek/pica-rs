#![no_main]

use std::str::FromStr;

use libfuzzer_sys::fuzz_target;
use pica_select::Query;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Query::from_str(s);
    }
});
