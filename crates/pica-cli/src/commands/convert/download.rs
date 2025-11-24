use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use bstr::ByteSlice;
use pica_record::io::{ReadPicaError, RecordsIter};
use pica_record::{ByteRecord, StringRecord};

pub(crate) struct DownloadReader {
    inner: BufReader<File>,
    buf: Vec<u8>,
}

impl DownloadReader {
    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> io::Result<Self> {
        let reader = File::open(path)?;

        Ok(Self {
            inner: BufReader::new(reader),
            buf: Vec::new(),
        })
    }
}

impl RecordsIter for DownloadReader {
    type ByteItem<'a>
        = Result<ByteRecord<'a>, ReadPicaError>
    where
        Self: 'a;

    type StringItem<'a>
        = Result<StringRecord<'a>, ReadPicaError>
    where
        Self: 'a;

    fn next_byte_record(&mut self) -> Option<Self::ByteItem<'_>> {
        self.buf.clear();

        match self.inner.read_until(b'\n', &mut self.buf) {
            Err(e) => return Some(Err(ReadPicaError::from(e))),
            Ok(0) => return None,
            Ok(_) => {
                if !self.buf.starts_with(b"SET:") {
                    return Some(Err(ReadPicaError::Other(
                        "expected line starting with phrase 'SET:'"
                            .into(),
                    )));
                }

                match self.inner.read_until(b'\n', &mut self.buf) {
                    Err(e) => return Some(Err(ReadPicaError::from(e))),
                    Ok(n) => {
                        if n != 2 {
                            return Some(Err(ReadPicaError::Other(
                                "expected empty line".into(),
                            )));
                        }
                    }
                }

                self.buf.clear();
            }
        }

        loop {
            match self.inner.read_until(b'\n', &mut self.buf) {
                Err(e) => return Some(Err(ReadPicaError::from(e))),
                Ok(n) => {
                    if n == 2 {
                        break;
                    }
                }
            }
        }

        self.buf = self
            .buf
            .replace(b"\xc6\x92", b"\x1f")
            .replace(b"\x0D\x0A", b"\x1E")
            .replace(b"\x1E\x1E", b"\x1E\x0A");

        match ByteRecord::from_bytes(&self.buf) {
            Ok(record) => Some(Ok(record)),
            Err(err) => Some(Err(ReadPicaError::Parse {
                msg: "invalid record".into(),
                err,
            })),
        }
    }
    fn next_string_record(&mut self) -> Option<Self::StringItem<'_>> {
        todo!()
    }
}
