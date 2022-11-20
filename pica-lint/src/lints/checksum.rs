use pica_path::{Path, PathExt};
use pica_record::ByteRecord;
use serde::Deserialize;

use super::{Lint, Status};

#[derive(Debug, Deserialize)]
pub struct Checksum {
    path: Path,
}

impl Lint for Checksum {
    fn check<'a>(&mut self, record: &ByteRecord) -> Status {
        for value in record.path(&self.path).iter() {
            let mut value =
                value.iter().map(|i| i - 48).collect::<Vec<u8>>();

            if !(8..=11).contains(&value.len()) {
                return Status::Hit;
            }

            let actual: u64 = value.pop().unwrap() as u64;
            let mut expected: u64 = value
                .into_iter()
                .rev()
                .zip(2..=11)
                .fold(0 as u64, |acc, (value, factor)| {
                    acc + (value as u64) * factor
                });

            expected = (11 - (expected % 11)) % 11;
            if expected == 10 {
                expected = 'X' as u64 - 48;
            }

            if expected != actual {
                return Status::Hit;
            }
        }

        Status::Miss
    }
}
