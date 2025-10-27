use hashbrown::HashMap;
use pica_record::prelude::*;

#[inline(always)]
const fn duplicates_threshold() -> usize {
    2
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Duplicates {
    query: Query,

    #[serde(default = "duplicates_threshold")]
    threshold: usize,

    #[serde(default = "super::strsim_threshold")]
    strsim_threshold: f64,

    #[serde(default)]
    case_ignore: bool,
}

impl Duplicates {
    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let options = QueryOptions::new()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        let mut freqs = HashMap::new();

        for row in record.query(&self.query, &options).iter() {
            let value = row
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("-");

            freqs
                .entry(value)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        freqs.retain(|_, cnt| *cnt >= self.threshold);
        if freqs.len() > 0 {
            let message = freqs
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            (true, Some(message))
        } else {
            (false, None)
        }
    }
}
