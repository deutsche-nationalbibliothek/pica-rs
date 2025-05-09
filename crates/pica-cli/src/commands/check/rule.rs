use std::io::Write;

use bstr::ByteSlice;
use pica_record::prelude::*;
use serde::{Deserialize, Serialize};

use super::checks::Checks;
use super::writer::Record;
use crate::prelude::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Level {
    #[default]
    Error,
    Warning,
    Info,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Rule {
    #[serde(skip)]
    pub(crate) id: String,

    #[allow(dead_code)]
    pub(crate) description: Option<String>,

    #[allow(dead_code)]
    pub(crate) link: Option<String>,

    #[serde(default)]
    pub(crate) level: Level,

    #[serde(flatten)]
    pub(crate) check: Checks,
}

impl Rule {
    #[inline(always)]
    pub(crate) fn preprocess(&mut self, record: &ByteRecord) {
        if let Checks::Link(ref mut c) = self.check {
            c.preprocess(record);
        }
    }

    pub(crate) fn check<W: Write>(
        &mut self,
        record: &ByteRecord,
        writer: &mut csv::Writer<W>,
    ) -> Result<(), CliError> {
        let (result, message) = match self.check {
            Checks::DateTime(ref c) => c.check(record),
            Checks::Filter(ref c) => c.check(record),
            Checks::Isni(ref c) => c.check(record),
            Checks::Iso639(ref c) => c.check(record),
            Checks::Link(ref mut c) => c.check(record),
            Checks::Unicode(ref c) => c.check(record),
        };

        if result {
            writer.serialize(Record {
                ppn: record.ppn(),
                rule: &self.id,
                level: &self.level,
                message,
            })?;
        }

        Ok(())
    }

    pub(crate) fn finish<W: Write>(
        &mut self,
        writer: &mut csv::Writer<W>,
    ) -> Result<(), CliError> {
        if let Checks::Link(ref mut c) = self.check {
            for (ppn, message) in c.finish() {
                let ppn = if !ppn.is_empty() {
                    Some(ppn.as_bstr())
                } else {
                    None
                };

                writer.serialize(Record {
                    rule: &self.id,
                    level: &self.level,
                    message,
                    ppn,
                })?;
            }
        }

        Ok(())
    }
}
