use std::fs::File;
use std::path::PathBuf;
use std::{io, path};

use bstr::{BString, ByteSlice};
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

#[derive(Debug, Default)]
pub(crate) struct FilterSetBuilder {
    column: Option<String>,
    path: Option<Path>,
    allow: Vec<PathBuf>,
    deny: Vec<PathBuf>,
}

impl FilterSetBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn column(mut self, column: Option<String>) -> Self {
        self.column = column;
        self
    }

    pub(crate) fn source(mut self, path: Option<Path>) -> Self {
        self.path = path;
        self
    }

    pub(crate) fn allow<P>(mut self, list: Vec<P>) -> Self
    where
        P: AsRef<path::Path>,
    {
        self.allow =
            list.iter().map(|p| PathBuf::from(p.as_ref())).collect();
        self
    }

    pub(crate) fn deny<P>(mut self, list: Vec<P>) -> Self
    where
        P: AsRef<path::Path>,
    {
        self.deny =
            list.iter().map(|p| PathBuf::from(p.as_ref())).collect();

        self
    }

    pub(crate) fn build(self) -> Result<FilterSet, FilterSetError> {
        let path = self.path.unwrap_or(Path::new("003@.0").unwrap());
        let options = MatcherOptions::default();

        let allow = read_filter_list(&self.allow, &self.column)?;
        let deny = read_filter_list(&self.deny, &self.column)?;

        Ok(FilterSet {
            allow,
            deny,
            path,
            options,
        })
    }
}

fn read_filter_list(
    paths: &[PathBuf],
    column: &Option<String>,
) -> Result<Option<HashSet<BString>>, FilterSetError> {
    if paths.is_empty() {
        return Ok(None);
    }

    let mut set = HashSet::new();

    for path in paths.iter() {
        let path_str = path.to_str().unwrap_or_default();
        let df = if path_str.ends_with("ipc") {
            IpcReader::new(File::open(path)?)
                .memory_mapped(None)
                .finish()?
        } else if path_str.ends_with("tsv")
            || path_str.ends_with("tsv.gz")
        {
            CsvReadOptions::default()
                .with_has_header(true)
                .with_infer_schema_length(Some(0))
                .with_parse_options(
                    CsvParseOptions::default().with_separator(b'\t'),
                )
                .try_into_reader_with_file_path(Some(path.into()))?
                .finish()?
        } else {
            CsvReadOptions::default()
                .with_has_header(true)
                .with_infer_schema_length(Some(0))
                .try_into_reader_with_file_path(Some(path.into()))?
                .finish()?
        };

        let column = if let Some(name) = column {
            df.column(name).map_err(|_| {
                FilterSetError::Other(format!(
                    "Missing column `{}` in file {}. ",
                    name,
                    path.display()
                ))
            })?
        } else {
            df.column("ppn").or(df.column("idn")).map_err(|_| {
                FilterSetError::Other(format!(
                    "Missing a column `ppn` or `idn` in file {}. ",
                    path.display()
                ))
            })?
        };

        set.extend(
            column
                .cast(&DataType::String)?
                .str()?
                .iter()
                .filter_map(|idn| idn.map(BString::from)),
        );
    }

    Ok(Some(set))
}

pub(crate) struct FilterSet {
    allow: Option<HashSet<BString>>,
    deny: Option<HashSet<BString>>,
    path: Path,
    options: MatcherOptions,
}

impl FilterSet {
    #[inline(always)]
    pub(crate) fn check(&self, record: &ByteRecord) -> bool {
        let values: Vec<_> =
            record.path(&self.path, &self.options).collect();

        if let Some(ref deny) = self.deny
            && (values.iter().any(|v| deny.contains(v.as_bstr())))
        {
            return false;
        }

        if let Some(ref allow) = self.allow
            && (!values.iter().any(|v| allow.contains(v.as_bstr()))
                || allow.is_empty())
        {
            return false;
        }

        true
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
