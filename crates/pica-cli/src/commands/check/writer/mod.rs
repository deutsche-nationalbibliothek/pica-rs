use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;

use bstr::BStr;

use super::rule::Level;

#[derive(Debug, serde::Serialize)]
pub(crate) struct Record<'a, 'b> {
    pub(crate) ppn: Option<&'a BStr>,
    pub(crate) rule: &'b str,
    pub(crate) level: &'b Level,
    pub(crate) message: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
}

pub(crate) struct CsvWriter {
    inner: csv::Writer<Box<dyn Write>>,
}

impl CsvWriter {
    pub(crate) fn from_path(
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let wtr: Box<dyn Write> = match path {
            Some(path) => Box::new(BufWriter::new(File::create(path)?)),
            None => Box::new(stdout().lock()),
        };

        let wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(wtr);

        Ok(Self { inner: wtr })
    }

    pub(crate) fn write_record(
        &mut self,
        record: Record,
    ) -> Result<(), Error> {
        Ok(self.inner.serialize(record)?)
    }

    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        Ok(self.inner.flush()?)
    }
}

pub(crate) struct TxtWriter {
    inner: csv::Writer<Box<dyn Write>>,
}

impl TxtWriter {
    pub(crate) fn from_path(
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let wtr: Box<dyn Write> = match path {
            Some(path) => Box::new(BufWriter::new(File::create(path)?)),
            None => Box::new(stdout().lock()),
        };

        let wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(wtr);

        Ok(Self { inner: wtr })
    }

    pub(crate) fn write_record(
        &mut self,
        record: Record,
    ) -> Result<(), Error> {
        let ppn = record.ppn.unwrap_or_default();
        Ok(self.inner.write_record([ppn])?)
    }

    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        Ok(self.inner.flush()?)
    }
}

pub(crate) enum Writer {
    Csv(CsvWriter),
    Txt(TxtWriter),
}

impl Writer {
    pub(crate) fn from_path(
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let path_str = if let Some(ref path) = path {
            path.to_str().unwrap_or_default()
        } else {
            "".into()
        };

        if path_str.ends_with(".txt") {
            Ok(Self::Txt(TxtWriter::from_path(path)?))
        } else {
            Ok(Self::Csv(CsvWriter::from_path(path)?))
        }
    }

    pub(crate) fn write_record(
        &mut self,
        record: Record,
    ) -> Result<(), Error> {
        match self {
            Self::Csv(wtr) => wtr.write_record(record),
            Self::Txt(wtr) => wtr.write_record(record),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        match self {
            Self::Csv(wtr) => wtr.finish(),
            Self::Txt(wtr) => wtr.finish(),
        }
    }
}
