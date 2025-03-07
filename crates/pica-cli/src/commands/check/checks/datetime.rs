use jiff::civil;
use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DateTime {
    path: Path,
    message: Option<String>,
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
        for value in record.path(&self.path, &Default::default()) {
            if self.offset >= value.len()
                || civil::DateTime::strptime(
                    &self.format,
                    &value[self.offset..],
                )
                .is_err()
            {
                let message = self
                    .message
                    .as_ref()
                    .map(|m| m.replace("{}", &value.to_string()));

                return (true, message);
            }
        }

        (false, None)
    }
}
