use std::path::PathBuf;
use std::slice;

use bstr::{BString, ByteSlice};
use hashbrown::HashSet;
use pica_record::prelude::*;

use crate::error::{CliError, bail};
use crate::utils::read_filter_list;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Allow {
    path: Path,
    list: List,

    #[serde(skip)]
    values: HashSet<BString>,

    #[serde(default = "super::strsim_threshold")]
    strsim_threshold: f64,

    #[serde(default)]
    case_ignore: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct List {
    source: PathBuf,
    column: String,
}

impl Allow {
    pub(crate) fn initialize(&mut self) -> Result<(), CliError> {
        let Some(values) = read_filter_list(
            slice::from_ref(&self.list.source),
            &Some(self.list.column.clone()),
        )?
        else {
            bail!("unable to read allow list");
        };

        self.values = values;
        Ok(())
    }

    pub(crate) fn check(
        &self,
        record: &ByteRecord,
    ) -> (bool, Option<String>) {
        let options = MatcherOptions::default()
            .strsim_threshold(self.strsim_threshold)
            .case_ignore(self.case_ignore);

        let message = record
            .path(&self.path, &options)
            .filter(|value| !self.values.contains(value.as_bstr()))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        if !message.is_empty() {
            (true, Some(message))
        } else {
            (false, None)
        }
    }
}
