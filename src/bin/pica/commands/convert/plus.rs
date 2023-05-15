use std::ffi::OsString;
use std::io;

use pica_record::io::{ByteRecordWrite, WriterBuilder};

pub(crate) struct PlusWriter {}

impl PlusWriter {
    pub(crate) fn new(
        output: Option<OsString>,
    ) -> io::Result<Box<dyn ByteRecordWrite>> {
        WriterBuilder::new().from_path_or_stdout(output)
    }
}
