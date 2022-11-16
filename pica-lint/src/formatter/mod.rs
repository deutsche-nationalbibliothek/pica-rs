use std::io;

use pica_record::ByteRecord;

mod csv;
pub use self::csv::CsvFormatter;
use crate::rules::Severity;

pub trait Formatter: Send + Sync {
    fn fmt(
        &mut self,
        id: &str,
        record: &ByteRecord,
        severity: &Severity,
    ) -> io::Result<()>;
    fn finish(&mut self) -> io::Result<()>;
}
