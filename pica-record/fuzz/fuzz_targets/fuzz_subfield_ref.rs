#![no_main]
use libfuzzer_sys::fuzz_target;
use pica_record::SubfieldRef;

fuzz_target!(|data: &[u8]| {
    let _subfield = SubfieldRef::from_bytes(data);
});
