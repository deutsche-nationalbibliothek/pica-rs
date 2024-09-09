use pica_matcher::{MatcherOptions, RecordMatcher};
use pica_record_v1::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Filter {
    filter: RecordMatcher,
    #[serde(default)]
    case_ignore: bool,
    #[serde(default)]
    invert: bool,
}

impl Lint for Filter {
    fn check(&mut self, record: &ByteRecord) -> Status {
        let options =
            MatcherOptions::new().case_ignore(self.case_ignore);

        let mut result = self.filter.is_match(record, &options);
        if self.invert {
            result = !result;
        }

        if result {
            return Status::Hit;
        }

        Status::Miss
    }
}
