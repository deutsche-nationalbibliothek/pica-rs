use pica_matcher::{MatcherOptions, RecordMatcher};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::Lint;

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
    fn check(&self, record: &ByteRecord) -> bool {
        let options =
            MatcherOptions::new().case_ignore(self.case_ignore);

        let mut result = self.filter.is_match(record, &options);
        if self.invert {
            result = !result;
        }

        result
    }
}
