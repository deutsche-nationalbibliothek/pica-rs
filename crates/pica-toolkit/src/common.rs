use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

use arrow2::array::Utf8Array;
use arrow2::datatypes::DataType;
use arrow2::error::Result;
use arrow2::io::ipc::read::{read_file_metadata, FileReader};
use bstr::BString;
use csv::ReaderBuilder;
use pica_path::PathExt;
use pica_record::ByteRecord;

use crate::util::{CliError, CliResult};

#[derive(Debug, Default)]
pub(crate) struct FilterList(BTreeSet<BString>);

impl Deref for FilterList {
    type Target = BTreeSet<BString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FilterList {
    pub(crate) fn new(filenames: Vec<PathBuf>) -> CliResult<Self> {
        let mut ids: BTreeSet<BString> = BTreeSet::new();

        for path in filenames {
            match path.extension().and_then(OsStr::to_str) {
                Some("arrow") | Some("ipc") | Some("feather") => {
                    let mut reader = File::open(path)?;
                    let metadata = read_file_metadata(&mut reader)
                        .map_err(|_| {
                            CliError::Other(
                                "Unable to read IPC metadata".into(),
                            )
                        })?;

                    let index = metadata
                        .schema
                        .fields
                        .iter()
                        .position(|f| f.name == "idn")
                        .ok_or_else(|| {
                            CliError::Other(
                                "Unable to find `idn` column.".into(),
                            )
                        })?;

                    let reader = FileReader::new(
                        reader,
                        metadata,
                        Some(vec![index]),
                        None,
                    );

                    let chunks = reader
                        .collect::<Result<Vec<_>>>()
                        .map_err(|_| {
                            CliError::Other(
                                "Unable to collect chunks.".into(),
                            )
                        })?;

                    for chunk in chunks {
                        let array = &chunk.columns().first().unwrap();
                        if array.data_type() != &DataType::Utf8 {
                            return Err(CliError::Other(
                                "Expected Utf8 datatype".into(),
                            ));
                        }

                        let rows = array
                            .as_any()
                            .downcast_ref::<Utf8Array<i32>>()
                            .ok_or_else(|| {
                                CliError::Other(
                                    "Unable to downcast array.".into(),
                                )
                            })?;

                        for idn in rows.values_iter() {
                            ids.insert(BString::from(idn));
                        }
                    }
                }
                _ => {
                    let mut reader = ReaderBuilder::new()
                        .has_headers(false)
                        .from_path(path)?;

                    for result in reader.byte_records() {
                        let row = result?;

                        ids.insert(BString::from(
                            row.get(0).expect("idn in column 1"),
                        ));
                    }
                }
            }
        }

        Ok(Self(ids))
    }

    pub(crate) fn check(&self, record: &ByteRecord) -> bool {
        if let Some(idn) = record.idn() {
            if self.contains(idn) {
                return true;
            }
        }

        false
    }
}
