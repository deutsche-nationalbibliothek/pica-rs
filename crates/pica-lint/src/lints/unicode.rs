use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Debug, Deserialize)]
pub struct Unicode {}

impl Lint for Unicode {
    fn check(&mut self, record: &ByteRecord) -> Status {
        record.validate().is_err().into()
    }
}
