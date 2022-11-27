use std::io::{BufReader, Read};

use super::ReadPicaError;
use crate::ByteRecord;

pub struct Reader<R: Read> {
    inner: BufReader<R>,
    buffer: Vec<u8>,
}

impl<R: Read> Reader<R> {
    /// ```rust
    /// use std::io::{Cursor, Seek};
    ///
    /// use pica_record::io::{Reader, RecordsIterator};
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let data =
    ///         Cursor::new(b"003@ \x1f0abc\x1e\n003@ \x1f0def\x1e\n");
    ///     let mut reader = Reader::from_reader(data);
    ///
    ///     let mut count = 0;
    ///     while let Some(result) = reader.next() {
    ///         count += 1;
    ///     }
    ///
    ///     assert_eq!(count, 0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn from_reader(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
            buffer: vec![],
        }
    }

    pub fn into_inner(self) -> BufReader<R> {
        self.inner
    }
}

pub trait RecordsIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

impl<R: Read> RecordsIterator for Reader<R> {
    type Item<'a> = Result<ByteRecord<'a>, ReadPicaError> where Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        None
    }
}
