use std::fmt::Debug;

use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::Lint;

#[derive(Deserialize, Debug)]
pub struct Iri {
    path: Path,
}

impl Lint for Iri {
    fn check(&self, record: &ByteRecord) -> bool {
        record
            .path(&self.path)
            .iter()
            .map(ToString::to_string)
            .any(|value| sophia::iri::Iri::new(&value).is_err())
    }
}
