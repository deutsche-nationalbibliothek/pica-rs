#![no_main]

use libfuzzer_sys::fuzz_target;
use pica_record_v1::RecordRef;

fuzz_target!(|data: &[u8]| {
    let _record = RecordRef::from_bytes(data);
});
