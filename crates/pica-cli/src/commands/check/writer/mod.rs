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
    CSV(#[from] csv::Error),
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

pub(crate) enum Writer {
    Csv(CsvWriter),
}

impl Writer {
    pub(crate) fn from_path(
        path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        Ok(Self::Csv(CsvWriter::from_path(path)?))
    }

    pub(crate) fn write_record(
        &mut self,
        record: Record,
    ) -> Result<(), Error> {
        match self {
            Self::Csv(wtr) => wtr.write_record(record),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        match self {
            Self::Csv(wtr) => wtr.finish(),
        }
    }
}

// pub(crate) trait Writer {
//     fn write_record(&mut self, record: &Record) -> Result<(), Error>;
//     fn flush(&mut self) -> Result<(), Error>;
//     fn finish(self) -> Result<(), Error>;
// }

// pub(crate) struct CsvWriter<W: Write> {
//     inner: csv::Writer<W>,
// }

// impl<W: Write> Writer for CsvWriter<W> {
//     fn write_record(&mut self, record: &Record) -> Result<(), Error>
// {         Ok(self.inner.serialize(record)?)
//     }

//     fn flush(&mut self) -> Result<(), Error> {
//         Ok(self.inner.flush()?)
//     }

//     fn finish(mut self) -> Result<(), Error> {
//         Ok(self.inner.flush()?)
//     }
// }

// pub(crate) fn writer(
//     output: OsString,
// ) -> Result<Box<dyn Writer>, Error> {
// }
