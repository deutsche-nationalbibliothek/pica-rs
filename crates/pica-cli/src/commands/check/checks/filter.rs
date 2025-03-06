use pica_record::prelude::*;

use crate::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Filter {
    #[serde(rename = "filter")]
    pub(crate) matcher: RecordMatcher,

    /// The threshold for string similarity comparisons.
    pub(crate) strsim_threshold: Option<f64>,

    /// Whether to ignore case when comparing strings or not.
    #[serde(default)]
    pub(crate) case_ignore: bool,

    /// Whther to find records that did not match
    #[serde(default)]
    pub(crate) invert_match: bool,
}

impl Filter {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
        _config: &Config,
    ) -> (bool, Option<String>) {
        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold.unwrap_or(0.8))
            .case_ignore(self.case_ignore);

        let mut result = self.matcher.is_match(record, &options);
        if self.invert_match {
            result = !result;
        }

        (result, None)
    }
}
