use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Deserialize, Debug)]
pub struct Orcid {
    path: Path,
    #[serde(default)]
    prefix: String,
}

impl Lint for Orcid {
    fn check(&mut self, record: &ByteRecord) -> Status {
        let values = record.path(&self.path);
        if !values.is_empty() {
            for value in values {
                if value.starts_with(&self.prefix.as_bytes()) {
                    let value = value
                        .strip_prefix(self.prefix.as_bytes())
                        .unwrap()
                        .iter()
                        .filter_map(|c| {
                            if *c >= 48 {
                                Some(c - 48)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<u8>>();

                    if value.len() != 16 {
                        return Status::Hit;
                    }

                    let mut total: u64 = 0;

                    for i in 0..=14 {
                        total = (total + value[i] as u64) * 2;
                    }

                    let remainder = total % 11;
                    let mut result = (12 - remainder) % 11;
                    if result == 10 {
                        result = 40;
                    }

                    if result != value[15] as u64 {
                        return Status::Hit;
                    }
                }
            }
        }

        Status::Miss
    }
}
