use std::io::Write;

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
    pub(crate) fn preprocess(&mut self, _record: &ByteRecord) {}

    pub(crate) fn check<W: Write>(
        &mut self,
        record: &ByteRecord,
        config: &Config,
        writer: &mut csv::Writer<W>,
    ) -> Result<(), CliError> {
        let (result, message) = match self.check {
            Checks::Filter(ref check) => check.check(record, config),
            Checks::Unicode(ref check) => check.check(record, config),
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
}
