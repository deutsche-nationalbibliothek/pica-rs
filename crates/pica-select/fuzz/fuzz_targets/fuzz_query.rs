#![no_main]

use libfuzzer_sys::fuzz_target;
use pica_select::Query;

fuzz_target!(|data: &[u8]| {
    let _ = Query::try_from(data);
});
