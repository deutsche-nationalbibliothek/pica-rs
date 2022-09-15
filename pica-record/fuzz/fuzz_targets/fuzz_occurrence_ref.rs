#![no_main]
use libfuzzer_sys::fuzz_target;
use pica_record::OccurrenceRef;

fuzz_target!(|data: &[u8]| {
    let _occurrence = OccurrenceRef::from_bytes(data);
});
