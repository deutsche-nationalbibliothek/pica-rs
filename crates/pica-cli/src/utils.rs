use std::fs::File;
use std::io;
use std::path::Path;

use bstr::{BStr, BString};
use hashbrown::HashSet;
use pica_record::matcher::ParseMatcherError;
use pica_record::prelude::*;
use polars::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilterSetError {
    #[error(transparent)]
    Polars(#[from] PolarsError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("{0}")]
    Other(String),
}

pub(crate) struct FilterSet {
    allow: Option<HashSet<BString>>,
    deny: Option<HashSet<BString>>,
}

impl FilterSet {
    pub(crate) fn new<P: AsRef<Path>>(
        allow: Vec<P>,
        deny: Vec<P>,
    ) -> Result<Self, FilterSetError> {
        let allow = read_filter_set(allow)?;
        let deny = read_filter_set(deny)?;

        Ok(Self { allow, deny })
    }

    #[inline]
    pub(crate) fn check(&self, ppn: &BStr) -> bool {
        if let Some(ref deny) = self.deny {
            if deny.contains(ppn) {
                return false;
            }
        }

        if let Some(ref allow) = self.allow {
            if !allow.contains(ppn) || allow.is_empty() {
                return false;
            }
        }

        true
    }
}

fn read_filter_set<P: AsRef<Path>>(
    paths: Vec<P>,
) -> Result<Option<HashSet<BString>>, FilterSetError> {
    let mut set = HashSet::new();

    for path in paths.iter() {
        let df = read_df(path)?;
        let column =
            df.column("ppn").or(df.column("idn")).map_err(|_| {
                FilterSetError::Other(format!(
                    "Missing a column `ppn` or `idn` in file {}. ",
                    path.as_ref().display()
                ))
            })?;

        set.extend(
            column
                .cast(&DataType::String)?
                .str()?
                .iter()
                .filter_map(|idn| idn.map(BString::from)),
        );
    }

    Ok(if !paths.is_empty() { Some(set) } else { None })
}

fn read_df<P: AsRef<Path>>(
    path: P,
) -> Result<DataFrame, FilterSetError> {
    let path = path.as_ref().to_path_buf();
    let path_str = path.to_str().unwrap_or_default();

    if path_str.ends_with("ipc") || path_str.ends_with("arrow") {
        Ok(IpcReader::new(File::open(path)?)
            .memory_mapped(None)
            .finish()?)
    } else if path_str.ends_with("tsv") || path_str.ends_with("tsv.gz")
    {
        Ok(CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0))
            .with_parse_options(
                CsvParseOptions::default().with_separator(b'\t'),
            )
            .try_into_reader_with_file_path(Some(path))?
            .finish()?)
    } else {
        Ok(CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0))
            .try_into_reader_with_file_path(Some(path))?
            .finish()?)
    }
}

pub(crate) fn parse_predicates<S: AsRef<str>>(
    predicates: Option<S>,
) -> Result<Vec<(TagMatcher, OccurrenceMatcher)>, ParseMatcherError> {
    let Some(predicates) = predicates else {
        return Ok(vec![]);
    };

    let mut result = vec![];
    let items = predicates
        .as_ref()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty());

    for item in items {
        if let Some(pos) = item.rfind('/') {
            result.push((
                TagMatcher::new(&item[0..pos])?,
                OccurrenceMatcher::new(&item[pos..])?,
            ));
        } else {
            result.push((
                TagMatcher::new(item)?,
                OccurrenceMatcher::None,
            ));
        }
    }

    Ok(result)
}
