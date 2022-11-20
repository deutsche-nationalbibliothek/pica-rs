use std::collections::{BTreeMap, BTreeSet};

use bstr::BString;
use pica_matcher::RecordMatcher;
use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RefCheck {
    src: Path,
    dst: Path,
    filter: Option<RecordMatcher>,
    #[serde(skip, default)]
    seen: BTreeSet<BString>,
    #[serde(skip, default)]
    unseen: BTreeMap<BString, Vec<BString>>,
}

impl Lint for RefCheck {
    fn preprocess(&mut self, record: &ByteRecord) {
        let values = record.path(&self.dst);
        if !values.is_empty() {
            let insert = if let Some(ref filter) = self.filter {
                filter.is_match(record, &Default::default())
            } else {
                true
            };

            if insert {
                for value in values {
                    self.seen.insert(value.to_owned().into());
                }
            }
        }
    }

    fn check(&mut self, record: &ByteRecord) -> Status {
        let values = record.path(&self.src);
        let mut status = Status::Miss;

        if !values.is_empty() {
            status = Status::Postponed;

            for value in values {
                let idn = record.idn().unwrap().to_owned();
                self.unseen
                    .entry(value.to_owned().into())
                    .and_modify(|e| e.push(idn.into()))
                    .or_insert(vec![idn.into()]);
            }
        }

        status
    }

    fn finish(&mut self) -> Vec<(BString, Status)> {
        let mut result = vec![];

        for (key, idns) in self.unseen.iter() {
            if !self.seen.contains(key) {
                for idn in idns {
                    result.push((idn.to_owned(), Status::Hit))
                }
            }
        }

        result
    }
}
