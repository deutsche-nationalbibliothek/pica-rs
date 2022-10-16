//! I/O utilities for reading PICA+ records.

use std::io;

use thiserror::Error;

use crate::{ByteRecord, ParsePicaError};

/// An error that can occur when reading PICA+ records from a
/// [BufReader](std::io::BufReader).
#[derive(Error, Debug)]
pub enum ReadPicaError {
    #[error("parse error")]
    Parse(#[from] ParsePicaError),

    #[error("io error")]
    Io(#[from] io::Error),
}

type ReadResult<T> = std::result::Result<T, ReadPicaError>;

/// An extension of [BufRead](`std::io::BufRead`) which provides a
/// convenience API for reading [ByteRecord](`crate::ByteRecord`)s.
pub trait BufReadExt: io::BufRead {
    fn for_pica_record<F>(
        &mut self,
        skip_invalid: bool,
        mut f: F,
    ) -> ReadResult<()>
    where
        F: FnMut(&ByteRecord) -> ReadResult<bool>,
    {
        let mut buf = vec![];

        'outer: loop {
            let num_bytes = self.read_until(b'\n', &mut buf)?;
            if num_bytes == 0 {
                break;
            }

            let result = ByteRecord::from_bytes(&buf);
            match result {
                Ok(record) => {
                    if f(&record)? == false {
                        break 'outer;
                    }
                }
                Err(_) if skip_invalid => (),
                Err(e) => return Err(ReadPicaError::from(e)),
            }

            buf.clear();
        }

        Ok(())
    }
}

impl<B: io::BufRead> BufReadExt for B {}
