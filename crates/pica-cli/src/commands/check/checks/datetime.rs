use jiff::civil;
use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DateTime {
    path: Path,

    #[serde(default = "super::strsim_threshold")]
    strsim_threshold: f64,

    #[serde(default)]
    case_ignore: bool,

    #[serde(default = "default_fmt")]
    format: String,

    #[serde(default)]
    offset: usize,
}

fn default_fmt() -> String {
    "%Y-%m-%d".into()
}

impl DateTime {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let mut messages = vec![];
        let mut retval = false;

        let options = MatcherOptions::default()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        for value in record.path(&self.path, &options) {
            if self.offset >= value.len()
                || civil::DateTime::strptime(
                    &self.format,
                    &value[self.offset..],
                )
                .is_err()
            {
                messages.push(value.to_string());
                retval = true;
            }
        }

        let message = if !messages.is_empty() {
            Some(messages.join(", "))
        } else {
            None
        };

        (retval, message)
    }
}
