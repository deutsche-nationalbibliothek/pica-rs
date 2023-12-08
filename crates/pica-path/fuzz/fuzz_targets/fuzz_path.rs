#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;
use pica_path::Path;

fuzz_target!(|data: &[u8]| {
    let _path = Path::try_from(data);
});
