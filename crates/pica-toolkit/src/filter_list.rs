use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use bstr::{BStr, BString};
use polars::prelude::*;

#[derive(Debug, Default)]
pub(crate) struct FilterList {
    allow: BTreeSet<BString>,
    deny: BTreeSet<BString>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum FilterListError {
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
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn check(&self, idn: Option<&BStr>) -> bool {
        if self.allow.is_empty() && self.deny.is_empty() {
            return true;
        }

        if let Some(idn) = idn {
            if !self.allow.is_empty() && !self.allow.contains(idn) {
                false
            } else {
                !self.deny.contains(idn)
            }
        } else {
            false
        }
    }

    pub(crate) fn allow(
        mut self,
        filenames: Vec<PathBuf>,
    ) -> Result<Self, FilterListError> {
        for path in filenames.iter() {
            self.allow.extend(
                self.read_df(path)?
                    .column("idn")
                    .map_err(|_| FilterListError::MissingColumn)?
                    .cast(&DataType::String)?
                    .str()?
                    .into_iter()
                    .filter_map(|idn| idn.map(BString::from)),
            );
        }

        Ok(self)
    }

    pub(crate) fn deny(
        mut self,
        filenames: Vec<PathBuf>,
    ) -> Result<Self, FilterListError> {
        for path in filenames.iter() {
            self.deny.extend(
                self.read_df(path)?
                    .column("idn")
                    .map_err(|_| FilterListError::MissingColumn)?
                    .cast(&DataType::String)?
                    .str()?
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
        let path = path.to_owned();

        let options = CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0));

        match extension {
            Some("ipc" | "arrow" | "feather") => {
                Ok(IpcReader::new(File::open(path)?)
                    .memory_mapped(None)
                    .finish()?)
            }
            Some("csv") => Ok(options
                .try_into_reader_with_file_path(Some(path))?
                .finish()?),
            Some("gz") if path_str.ends_with(".csv.gz") => Ok(options
                .try_into_reader_with_file_path(Some(path))?
                .finish()?),
            Some("tsv") => Ok(options
                .with_parse_options(
                    CsvParseOptions::default().with_separator(b'\t'),
                )
                .try_into_reader_with_file_path(Some(path))?
                .finish()?),
            Some("gz") if path_str.ends_with(".tsv.gz") => Ok(options
                .with_parse_options(
                    CsvParseOptions::default().with_separator(b'\t'),
                )
                .try_into_reader_with_file_path(Some(path))?
                .finish()?),
            _ => {
                Err(FilterListError::InvalidFileFormat(path_str.into()))
            }
        }
    }
}
