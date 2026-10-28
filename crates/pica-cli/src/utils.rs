use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{io, path};

use bstr::{BString, ByteSlice};
use flate2::read::GzDecoder;
use hashbrown::HashSet;
use pica_record::matcher::ParseMatcherError;
use pica_record::prelude::*;
use thiserror::Error;

use crate::cli::FilterOpts;

#[derive(Debug, Error)]
pub(crate) enum FilterSetError {
    #[error(transparent)]
    Csv(#[from] csv::Error),
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

    pub(crate) fn column<S>(mut self, column: Option<S>) -> Self
    where
        S: AsRef<str>,
    {
        self.column = column.map(|s| s.as_ref().to_owned());
        self
    }

    pub(crate) fn source(mut self, path: Option<&Path>) -> Self {
        self.path = path.cloned();
        self
    }

    pub(crate) fn allow<P>(mut self, list: &[P]) -> Self
    where
        P: AsRef<path::Path>,
    {
        self.allow =
            list.iter().map(|p| PathBuf::from(p.as_ref())).collect();
        self
    }

    pub(crate) fn deny<P>(mut self, list: &[P]) -> Self
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

impl TryFrom<&FilterOpts> for FilterSet {
    type Error = FilterSetError;

    fn try_from(opts: &FilterOpts) -> Result<Self, Self::Error> {
        FilterSetBuilder::new()
            .source(opts.filter_set_source.as_ref())
            .column(opts.filter_set_column.as_ref())
            .allow(&opts.allow)
            .deny(&opts.deny)
            .build()
    }
}

pub(crate) fn read_filter_list(
    paths: &[PathBuf],
    column: &Option<String>,
) -> Result<Option<HashSet<BString>>, FilterSetError> {
    if paths.is_empty() {
        return Ok(None);
    }

    let column = column.clone().unwrap_or("ppn".into());
    let mut set = HashSet::new();
    let mut index = 0;

    for path in paths.iter() {
        let path_str = path.to_str().unwrap_or_default();
        let delimiter = if path_str.ends_with("tsv")
            || path_str.ends_with("tsv.gz")
        {
            b'\t'
        } else {
            b','
        };

        let has_headers = !(path_str.ends_with(".txt")
            || path_str.ends_with(".txt.gz"));

        let inner: Box<dyn Read> = if path_str.ends_with(".gz") {
            Box::new(GzDecoder::new(File::open(path)?))
        } else {
            Box::new(BufReader::new(File::open(path)?))
        };

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .delimiter(delimiter)
            .from_reader(inner);

        if rdr.has_headers() {
            let mut found = false;
            for (i, header) in rdr.headers()?.iter().enumerate() {
                if header == column {
                    found = true;
                    index = i;
                    break;
                }
            }

            if !found {
                return Err(FilterSetError::Other(format!(
                    "could not find column '{column}' in filter set '{path_str}'"
                )));
            }
        }

        for result in rdr.byte_records() {
            if let Some(value) = result?.get(index) {
                set.insert(value.into());
            } else {
                return Err(FilterSetError::Other(format!(
                    "could not access value at index {index}"
                )));
            }
        }
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
