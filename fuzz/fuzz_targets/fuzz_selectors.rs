#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate pica;

use pica::Selectors;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _selectors = Selectors::decode(s);
    }
});
