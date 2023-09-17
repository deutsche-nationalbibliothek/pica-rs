use bstr::BString;
use pica_record::ByteRecord;
use serde::Deserialize;

use super::level::Level;
use crate::lints::{Lint, Lints, Status};
// use crate::stats::Stats;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Rule {
    #[serde(skip)]
    pub id: String,
    #[serde(default)]
    pub level: Level,
    #[serde(default)]
    pub description: String,
    pub lint: Lints,
}

impl Rule {
    pub fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    pub fn preprocess(&mut self, record: &ByteRecord) {
        self.lint.preprocess(record)
    }

    pub fn process(&mut self, record: &ByteRecord) -> Status {
        self.lint.check(record)
    }

    pub fn finish(&mut self) -> Vec<(BString, Status)> {
        self.lint.finish()
    }
}
