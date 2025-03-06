use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;

use hashbrown::HashMap;
use pica_record::prelude::*;

use super::rule::Rule;
use crate::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub(crate) struct RuleSet {
    pub(crate) scope: Option<RecordMatcher>,

    #[serde(rename = "rule", default)]
    pub(crate) rules: HashMap<String, Rule>,
}

impl RuleSet {
    pub(crate) fn from_path<P>(path: P) -> Result<Self, CliError>
    where
        P: AsRef<Path>,
    {
        let mut rs: Self = toml::from_str(&read_to_string(&path)?)
            .map_err(|e| {
                let filename = path.as_ref().to_string_lossy();
                CliError::Other(format!(
                    "invalid rule-set {filename}: {e}"
                ))
            })?;

        for (id, rule) in rs.rules.iter_mut() {
            rule.id = id.to_owned();
        }

        Ok(rs)
    }

    pub(crate) fn preprocess(&mut self, record: &ByteRecord) {
        self.rules
            .iter_mut()
            .for_each(|(_, rule)| rule.preprocess(record));
    }

    pub(crate) fn check<W: Write>(
        &mut self,
        record: &ByteRecord,
        config: &Config,
        writer: &mut csv::Writer<W>,
    ) -> Result<(), CliError> {
        if let Some(ref matcher) = self.scope {
            if !matcher.is_match(record, &Default::default()) {
                return Ok(());
            }
        }

        for (_, rule) in self.rules.iter_mut() {
            rule.check(record, config, writer)?;
        }

        Ok(())
    }
}
