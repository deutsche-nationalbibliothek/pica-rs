//! This module contains data structures and functions to parse and work with
//! bibliographic records encoded in PICA+.

mod borrowed;
pub mod legacy;
mod parser;

pub use borrowed::{Field, Occurrence, Record, Subfield};
pub(crate) use parser::{
    parse_field_occurrence, parse_field_tag, parse_record, parse_subfield_code,
    ParseResult,
};
