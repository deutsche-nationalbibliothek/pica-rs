use std::fmt::Debug;

use chrono::NaiveDate;
use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Deserialize, Debug)]
pub struct Date {
    path: Path,
    #[serde(default = "default_fmt")]
    format: String,
    #[serde(default)]
    offset: usize,
}

fn default_fmt() -> String {
    "%Y-%m-%d".to_string()
}

impl Lint for Date {
    fn check(&mut self, record: &ByteRecord) -> Status {
        let result = record
            .path(&self.path, &Default::default())
            .iter()
            .map(ToString::to_string)
            .any(|value| {
                self.offset >= value.len()
                    || NaiveDate::parse_from_str(
                        &value[self.offset..],
                        &self.format,
                    )
                    .is_err()
            });

        if result {
            Status::Hit
        } else {
            Status::Miss
        }
    }
}
