use bstr::{BString, ByteSlice};
use hashbrown::{HashMap, HashSet};
use pica_record::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Link {
    /// A path expression whose values are used to match the source
    /// values.
    destination: Path,

    /// A path expression whose values refer to the target values.
    source: Path,

    /// An additional condition that the destintion record must
    /// fulfill.
    condition: Option<RecordMatcher>,

    /// The minimum score for string similarity comparisons
    #[serde(default = "strsim_threshold")]
    strsim_threshold: f64,

    /// If set, comparison operations will be search case insensitive
    #[serde(default)]
    case_ignore: bool,

    #[serde(skip, default)]
    unseen: HashMap<BString, Vec<BString>>,

    #[serde(skip, default)]
    seen: HashSet<BString>,
}

const fn strsim_threshold() -> f64 {
    0.8
}

impl Link {
    pub(crate) fn preprocess(&mut self, record: &ByteRecord) {
        let options = MatcherOptions::default()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        let values = record
            .path(&self.destination, &options)
            .collect::<Vec<_>>();

        if !values.is_empty() {
            let insert = if let Some(ref m) = self.condition {
                m.is_match(record, &options)
            } else {
                true
            };

            if insert {
                for value in values {
                    self.seen.insert(value.to_owned());
                }
            }
        }
    }

    pub(crate) fn check(
        &mut self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let ppn = record.ppn().unwrap_or_default();
        let options = MatcherOptions::default()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        for value in record.path(&self.source, &options) {
            if self.seen.contains(value) {
                continue;
            }

            self.unseen
                .entry(ppn.to_owned())
                .and_modify(|entry| entry.push(value.to_owned()))
                .or_insert_with(|| vec![value.to_owned()]);
        }

        (false, None)
    }

    pub(crate) fn finish(&mut self) -> Vec<(BString, Option<String>)> {
        let mut result = vec![];

        for (ppn, values) in self.unseen.iter() {
            let message = values
                .iter()
                .filter(|k| !self.seen.contains(k.as_bstr()))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            if !message.is_empty() {
                result.push((ppn.to_owned(), Some(message)));
            }
        }

        result
    }
}
