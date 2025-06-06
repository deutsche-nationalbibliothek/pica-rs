#![no_main]

use libfuzzer_sys::fuzz_target;
use pica_record::prelude::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Path::new(s);
    }
});
