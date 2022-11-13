use std::fmt::Debug;

use chrono::NaiveDate;
use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::Lint;

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
    fn check(&self, record: &ByteRecord) -> bool {
        record.path(&self.path).iter().map(ToString::to_string).any(
            |value| {
                NaiveDate::parse_from_str(
                    &value[self.offset..],
                    &self.format,
                )
                .is_err()
            },
        )
    }
}
