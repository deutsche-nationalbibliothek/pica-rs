#![no_main]
use libfuzzer_sys::fuzz_target;
use pica_record::TagRef;

fuzz_target!(|data: &[u8]| {
    let _tag = TagRef::from_bytes(data);
});
