use pica_record::ByteRecord;

mod checksum;
mod date;
mod filter;
mod iri;
mod iso639;
mod unicode;

pub use checksum::Checksum;
pub use date::Date;
pub use filter::Filter;
pub use iri::Iri;
pub use iso639::Iso639;
pub use unicode::Unicode;

pub trait Lint {
    fn check(&self, record: &ByteRecord) -> bool;
}
