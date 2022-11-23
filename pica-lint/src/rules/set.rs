use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::Result;
use std::path::Path;

use bstr::ByteSlice;
use pica_matcher::RecordMatcher;
use pica_path::PathExt;
use pica_record::ByteRecord;
use serde::Deserialize;

use super::rule::Rule;
use crate::formatter::Formatter;
use crate::lints::Status;
use crate::rules::Level;
use crate::stats::Stats;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RuleSet {
    pub organizational_unit: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub scope: Option<RecordMatcher>,
    #[serde(rename = "rule")]
    pub rules: HashMap<String, Rule>,
}

impl RuleSet {
    /// Create a new rule set from a spec file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut rs: RuleSet = toml::from_str(&read_to_string(path)?)?;

        for (id, rule) in rs.rules.iter_mut() {
            rule.set_id(id);
        }

        Ok(rs)
    }

    pub fn preprocess(&mut self, record: &ByteRecord) {
        self.rules
            .iter_mut()
            .for_each(|(_, rule)| rule.preprocess(record));
    }

    pub fn check(
        &mut self,
        record: &ByteRecord,
        fmt: &mut Box<dyn Formatter>,
    ) -> Stats {
        let mut stats = Stats::new();

        if let Some(ref scope) = self.scope {
            if !scope.is_match(record, &Default::default()) {
                return stats;
            }
        }

        let rules: Vec<&Rule> = self
            .rules
            .iter_mut()
            .filter_map(|(_, rule)| {
                if rule.process(record) == Status::Hit {
                    Some(&*rule)
                } else {
                    None
                }
            })
            .collect();

        for rule in rules.iter() {
            fmt.fmt(rule, record.idn().unwrap()).unwrap();
            match rule.level {
                Level::Error => stats.errors += 1,
                Level::Warning => stats.warnings += 1,
                Level::Info => stats.infos += 1,
            }
        }

        stats.checks += self.rules.len() as u64;
        stats.records += 1;
        stats
    }

    pub fn finish(&mut self, fmt: &mut Box<dyn Formatter>) -> Stats {
        let mut stats = Stats::new();

        for (_, rule) in self.rules.iter_mut() {
            for (idn, status) in rule.finish().iter() {
                if *status == Status::Hit {
                    fmt.fmt(rule, idn.as_bstr()).unwrap();
                    match rule.level {
                        Level::Error => stats.errors += 1,
                        Level::Warning => stats.warnings += 1,
                        Level::Info => stats.infos += 1,
                    }
                }
            }
        }

        stats
    }
}
