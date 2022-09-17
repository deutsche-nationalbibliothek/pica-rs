#![no_main]
use libfuzzer_sys::fuzz_target;
use pica_record::FieldRef;

fuzz_target!(|data: &[u8]| {
    let _field = FieldRef::from_bytes(data);
});
