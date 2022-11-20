use std::io;

use bstr::BStr;

mod csv;
pub use self::csv::CsvFormatter;
use crate::rules::Rule;

pub trait Formatter {
    fn fmt(&mut self, rule: &Rule, idn: &BStr) -> io::Result<()>;
    fn finish(&mut self) -> io::Result<()>;
}
