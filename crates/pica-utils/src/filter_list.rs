use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use bstr::{BStr, BString};
use polars::prelude::*;

#[derive(Debug)]
pub struct FilterList {
    allow: BTreeSet<BString>,
    deny: BTreeSet<BString>,
}

#[derive(Debug, thiserror::Error)]
pub enum FilterListError {
    #[error("invalid file format (path = '{0}')")]
    InvalidFileFormat(String),

    #[error("missing 'idn' column")]
    MissingColumn,

    #[error(transparent)]
    Polars(#[from] PolarsError),

    #[error(transparent)]
    IO(#[from] io::Error),
}

impl FilterList {
    pub fn new() -> Self {
        Self {
            allow: BTreeSet::new(),
            deny: BTreeSet::new(),
        }
    }

    pub fn check(&self, idn: Option<&BStr>) -> bool {
        if self.allow.is_empty() && self.deny.is_empty() {
            return true;
        }

        if let Some(idn) = idn {
            if !self.allow.is_empty() && !self.allow.contains(idn) {
                false
            } else if self.deny.contains(idn) {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn allow(
        mut self,
        filenames: Vec<PathBuf>,
    ) -> Result<Self, FilterListError> {
        for path in filenames.iter() {
            self.allow.extend(
                self.read_df(path)?
                    .column("idn")
                    .map_err(|_| FilterListError::MissingColumn)?
                    .cast(&DataType::Utf8)?
                    .utf8()?
                    .into_iter()
                    .filter_map(|idn| idn.map(BString::from)),
            );
        }

        Ok(self)
    }

    pub fn deny(
        mut self,
        filenames: Vec<PathBuf>,
    ) -> Result<Self, FilterListError> {
        for path in filenames.iter() {
            self.deny.extend(
                self.read_df(path)?
                    .column("idn")
                    .map_err(|_| FilterListError::MissingColumn)?
                    .cast(&DataType::Utf8)?
                    .utf8()?
                    .into_iter()
                    .filter_map(|idn| idn.map(BString::from)),
            );
        }

        Ok(self)
    }

    fn read_df(
        &self,
        path: &PathBuf,
    ) -> Result<DataFrame, FilterListError> {
        let extension = path.extension().and_then(OsStr::to_str);
        let path_str = path.to_str().unwrap_or_default();

        let mut schema = Schema::new();
        schema.with_column("idn".into(), DataType::Utf8);

        match extension {
            Some("ipc" | "arrow" | "feather") => {
                Ok(IpcReader::new(File::open(path)?)
                    .memory_mapped(false)
                    .finish()?)
            }
            Some("csv") => Ok(CsvReader::from_path(path)?
                .with_schema(Some(Arc::new(schema)))
                .truncate_ragged_lines(true)
                .has_header(true)
                .finish()?),
            Some("gz") if path_str.ends_with(".csv.gz") => {
                Ok(CsvReader::from_path(path)?
                    .with_schema(Some(Arc::new(schema)))
                    .truncate_ragged_lines(true)
                    .has_header(true)
                    .finish()?)
            }
            Some("tsv") => Ok(CsvReader::from_path(path)?
                .with_separator(b'\t')
                .with_schema(Some(Arc::new(schema)))
                .truncate_ragged_lines(true)
                .has_header(true)
                .finish()?),
            Some("gz") if path_str.ends_with(".tsv.gz") => {
                Ok(CsvReader::from_path(path)?
                    .with_separator(b'\t')
                    .with_schema(Some(Arc::new(schema)))
                    .truncate_ragged_lines(true)
                    .has_header(true)
                    .finish()?)
            }
            _ => {
                return Err(FilterListError::InvalidFileFormat(
                    path_str.into(),
                ))
            }
        }
    }
}
