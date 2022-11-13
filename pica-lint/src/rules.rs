#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

use pica_matcher::RecordMatcher;
use pica_record::ByteRecord;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;

use crate::formatter::Formatter;
use crate::lints::*;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub id: String,
    pub scope: Option<RecordMatcher>,
    #[serde(rename = "rule")]
    pub rules: HashMap<String, Rule>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Rule {
    check: Check,
    #[serde(default)]
    severity: Severity,
    #[serde(default)]
    description: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "lint")]
pub enum Check {
    Checksum(Checksum),
    Date(Date),
    Filter(Filter),
    Iri(Iri),
    Iso639(Iso639),
    Unicode(Unicode),
}

impl Check {
    fn eval(&self, record: &ByteRecord) -> bool {
        match self {
            Check::Checksum(ref c) => c.check(record),
            Check::Date(ref c) => c.check(record),
            Check::Filter(ref c) => c.check(record),
            Check::Iri(ref c) => c.check(record),
            Check::Iso639(ref c) => c.check(record),
            Check::Unicode(ref c) => c.check(record),
        }
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    #[default]
    Error,
    Warning,
    Info,
}

impl Manifest {
    pub fn from_path(path: &PathBuf) -> io::Result<Self> {
        Ok(toml::from_str(&read_to_string(path)?)?)
    }

    pub fn check(
        &self,
        record: &ByteRecord,
        fmt: &mut Box<dyn Formatter>,
    ) -> anyhow::Result<(usize, usize, usize)> {
        let results: Vec<(String, Severity)> = self
            .rules
            .par_iter()
            .filter_map(|(id, rule)| {
                if rule.check.eval(record) {
                    Some((id.to_string(), rule.severity.clone()))
                } else {
                    None
                }
            })
            .collect();

        let mut warnings = 0;
        let mut errors = 0;
        let mut infos = 0;

        for (id, severity) in results.iter() {
            fmt.fmt(id, record)?;
            match severity {
                Severity::Warning => warnings += 1,
                Severity::Error => errors += 1,
                Severity::Info => infos += 1,
            }
        }

        Ok((errors, warnings, infos))
    }
}
