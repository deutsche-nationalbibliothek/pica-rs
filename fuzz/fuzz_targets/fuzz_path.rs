#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate pica;
use pica::Path;

fuzz_target!(|data: &[u8]| {
    let _path = Path::from_bytes(data);
});
