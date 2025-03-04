use std::io::Write;

use pica_record::prelude::*;

use super::checks::Checks;
use super::writer::Record;
use crate::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub(crate) struct Rule {
    #[serde(skip)]
    pub(crate) id: String,

    #[serde(flatten)]
    pub(crate) check: Checks,
}

impl Rule {
    #[inline(always)]
    pub(crate) fn preprocess(&mut self, _record: &ByteRecord) {}

    #[inline(always)]
    pub(crate) fn check<W: Write>(
        &mut self,
        record: &ByteRecord,
        writer: &mut csv::Writer<W>,
    ) -> Result<(), CliError> {
        let (result, comment) = match self.check {
            Checks::Unicode(ref check) => check.check(record),
        };

        if result {
            writer.serialize(Record {
                ppn: record.ppn(),
                rule: &self.id,
                comment,
            })?;
        }

        Ok(())
    }
}
