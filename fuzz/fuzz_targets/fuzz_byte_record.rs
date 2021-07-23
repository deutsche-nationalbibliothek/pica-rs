#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate pica;
use pica::ByteRecord;

fuzz_target!(|data: &[u8]| {
    let _record = ByteRecord::from_bytes(data);
});
