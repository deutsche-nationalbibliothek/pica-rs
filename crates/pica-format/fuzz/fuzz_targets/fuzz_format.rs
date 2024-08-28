#![no_main]

use std::str::FromStr;

use libfuzzer_sys::fuzz_target;
use pica_format::Format;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _format = Format::from_str(s);
    }
});
