//! This module contains data structures and functions to parse and work with
//! bibliographic records encoded in PICA+.

mod borrowed;
pub mod legacy;
mod parser;

pub use borrowed::{Field, Occurrence, Record, Subfield};
pub use parser::parse_record;
