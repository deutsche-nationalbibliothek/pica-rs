use std::fmt::Debug;

use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Deserialize, Debug)]
pub struct Iri {
    path: Path,
}

impl Lint for Iri {
    fn check(&mut self, record: &ByteRecord) -> Status {
        record
            .path(&self.path, &Default::default())
            .iter()
            .map(ToString::to_string)
            .any(|value| sophia::iri::Iri::new(&value).is_err())
            .into()
    }
}
