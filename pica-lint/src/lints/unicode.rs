use pica_record::ByteRecord;
use serde::Deserialize;

use super::Lint;

#[derive(Debug, Deserialize)]
pub struct Unicode {}

impl Lint for Unicode {
    fn check(&self, record: &ByteRecord) -> bool {
        record.validate().is_err()
    }
}
