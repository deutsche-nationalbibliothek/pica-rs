//! I/O utilities for reading PICA+ records.

use std::io;

use thiserror::Error;

use crate::parser::LF;
use crate::{ByteRecord, ParsePicaError};

type ParseResult<'a> = Result<ByteRecord<'a>, ParsePicaError>;
type ReadResult<T> = Result<T, ReadPicaError>;

/// An error that can occur when reading PICA+ records from a
/// [BufReader](std::io::BufReader).
#[derive(Error, Debug)]
pub enum ReadPicaError {
    #[error("parse error")]
    Parse(#[from] ParsePicaError),

    #[error("io error")]
    Io(#[from] io::Error),
}

/// An extension of [BufRead](`std::io::BufRead`) which provides a
/// convenience API for reading [ByteRecord](`crate::ByteRecord`)s.
pub trait BufReadExt: io::BufRead {
    /// Executes the given closure on each parsed line in the underlying
    /// reader.
    ///
    /// If the underlying reader or the closure returns an error, then
    /// the iteration stops and the error is returned. If the closure
    /// returns `false` the iteration is stopped and no error is
    /// returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::{Cursor, Seek};
    ///
    /// use pica_record::io::BufReadExt;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let mut reader =
    ///         Cursor::new(b"003@ \x1f0abc\x1e\n003@ \x1f0def\x1e\n");
    ///
    ///     // iterate over all records
    ///     let mut count = 0;
    ///     reader.for_pica_record(|result| {
    ///         let _record = result?;
    ///         count += 1;
    ///         Ok(true)
    ///     })?;
    ///
    ///     assert_eq!(count, 2);
    ///
    ///     // stop iteration after first record
    ///     reader.rewind()?;
    ///     count = 0;
    ///     reader.for_pica_record(|result| {
    ///         let _record = result?;
    ///         count += 1;
    ///         Ok(false)
    ///     })?;
    ///
    ///     assert_eq!(count, 1);
    ///
    ///     Ok(())
    /// }
    /// ```
    fn for_pica_record<F>(&mut self, mut f: F) -> ReadResult<()>
    where
        F: FnMut(ParseResult) -> ReadResult<bool>,
    {
        let mut buf = vec![];

        loop {
            let num_bytes = self.read_until(LF, &mut buf)?;
            if num_bytes == 0 {
                break;
            }

            let result = ByteRecord::from_bytes(&buf);
            if let Ok(false) = f(result) {
                break;
            }

            buf.clear();
        }

        Ok(())
    }
}

impl<B: io::BufRead> BufReadExt for B {}
