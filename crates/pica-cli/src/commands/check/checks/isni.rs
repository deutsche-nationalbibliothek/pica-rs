use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Isni {
    path: Path,
    prefix: Option<String>,
    strsim_threshold: Option<f64>,
    #[serde(default)]
    case_ignore: bool,
}

impl Isni {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let mut messages = vec![];
        let mut retval = false;

        let options = MatcherOptions::default()
            .strsim_threshold(self.strsim_threshold.unwrap_or(0.8))
            .case_ignore(self.case_ignore);

        for value in record.path(&self.path, &options) {
            let iter = if let Some(ref prefix) = self.prefix {
                if !value.starts_with(prefix.as_bytes()) {
                    continue;
                }

                value.strip_prefix(prefix.as_bytes()).unwrap().iter()
            } else {
                value.iter()
            };

            let digits: Vec<u8> =
                iter.filter_map(|c| {
                    if *c >= 48 { Some(c - 48) } else { None }
                })
                .collect();

            if digits.len() != 16 {
                messages.push(value.to_string());
                retval = true;
                continue;
            }

            let total = digits[0..15]
                .iter()
                .fold(0_u64, |acc, item| (acc + *item as u64) * 2);

            let reminder = total % 11;
            let mut result = (12 - reminder) % 11;
            if result == 10 {
                result = 40;
            }

            if result != digits[15] as u64 {
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
