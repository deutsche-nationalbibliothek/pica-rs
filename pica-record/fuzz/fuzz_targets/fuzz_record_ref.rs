#![no_main]
use libfuzzer_sys::fuzz_target;
use pica_record::RecordRef;

fuzz_target!(|data: &[u8]| {
    let _record = RecordRef::from_bytes(data);
});
