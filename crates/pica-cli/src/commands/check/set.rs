use std::fs::read_to_string;
use std::path::Path;

use hashbrown::HashMap;
use pica_record::prelude::*;
use unicode_normalization::UnicodeNormalization;

use super::rule::Rule;
use crate::commands::check::writer::Writer;
use crate::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RuleSet {
    pub(crate) scope: Option<RecordMatcher>,

    #[serde(default)]
    pub(crate) termination: Termination,

    #[serde(rename = "rule", default)]
    pub(crate) rules: HashMap<String, Rule>,
}

#[derive(Debug, PartialEq, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Termination {
    #[default]
    Default,
    Fast,
}

impl RuleSet {
    pub(crate) fn new<P>(
        path: P,
        nf: Option<&NormalizationForm>,
    ) -> Result<Self, CliError>
    where
        P: AsRef<Path>,
    {
        use NormalizationForm::*;

        let content = read_to_string(&path)?;
        let content = match nf {
            Some(Nfc) => content.chars().nfc().to_string(),
            Some(Nfkc) => content.chars().nfkc().to_string(),
            Some(Nfd) => content.chars().nfd().to_string(),
            Some(Nfkd) => content.chars().nfkd().to_string(),
            None => content,
        };

        let mut rs: Self = toml::from_str(&content).map_err(|e| {
            let filename = path.as_ref().to_string_lossy();
            CliError::Other(format!("invalid rule-set {filename}: {e}"))
        })?;

        for (id, rule) in rs.rules.iter_mut() {
            rule.id = id.to_owned();
            rule.initialize()?
        }

        Ok(rs)
    }

    pub(crate) fn preprocess(&mut self, record: &ByteRecord) {
        self.rules
            .iter_mut()
            .for_each(|(_, rule)| rule.preprocess(record));
    }

    pub(crate) fn check(
        &mut self,
        record: &ByteRecord,
        writer: &mut Writer,
    ) -> Result<(), CliError> {
        if let Some(ref matcher) = self.scope
            && !matcher.is_match(record, &Default::default())
        {
            return Ok(());
        }

        for (_, rule) in self.rules.iter_mut() {
            let result = rule.check(record, writer)?;

            match self.termination {
                Termination::Fast if result => break,
                _ => continue,
            }
        }

        Ok(())
    }

    pub(crate) fn finish(
        &mut self,
        writer: &mut Writer,
    ) -> Result<(), CliError> {
        for (_, rule) in self.rules.iter_mut() {
            rule.finish(writer)?;
        }

        Ok(())
    }
}
