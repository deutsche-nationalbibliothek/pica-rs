use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Filter {
    #[serde(rename = "filter")]
    matcher: RecordMatcher,

    #[serde(default = "super::strsim_threshold")]
    strsim_threshold: f64,

    #[serde(default)]
    case_ignore: bool,

    #[serde(default)]
    invert_match: bool,
}

impl Filter {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let options = MatcherOptions::new()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        let mut result = self.matcher.is_match(record, &options);
        if self.invert_match {
            result = !result;
        }

        (result, None)
    }
}
