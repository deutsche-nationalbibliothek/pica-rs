use std::ffi::OsString;
use std::fs::File;
use std::io::{BufWriter, Write, stdout};

use bstr::BStr;

use super::rule::Level;
use crate::prelude::*;

#[derive(Debug, serde::Serialize)]
pub(crate) struct Record<'a, 'b> {
    pub(crate) ppn: Option<&'a BStr>,
    pub(crate) rule: &'b str,
    pub(crate) level: &'b Level,
    pub(crate) message: Option<String>,
}

pub(crate) fn writer(
    output: Option<OsString>,
) -> Result<csv::Writer<Box<dyn Write>>, CliError> {
    let inner: Box<dyn Write> = if let Some(path) = output {
        Box::new(BufWriter::new(File::create(path)?))
    } else {
        Box::new(BufWriter::new(stdout().lock()))
    };

    Ok(csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(inner))
}
