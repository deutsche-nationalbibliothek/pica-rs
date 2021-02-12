//! This crate provides types and parsers to work with bibliographic records
//! encoded in PICA+.
//!
//! A PICA+ record consists of an non-empty list of [`Field`]s, which contains
//! a list of [`Subfield`]s. A PICA+ is defined as followed:
//!
//! ```text
//! <record>           ::= <field>+
//! <field>            ::= <field-name> <field-occurrence> <sp> <subfield>* <rs>
//! <field-name>       ::= [0-2] [0-9]{2} ([A-Z] | "@")
//! <field-occurrence> ::= "/" [0-9]{2,3}
//! <subfield>         ::= <us> <subfield-name> <subfield-value>
//! <subfield-name>    ::= [0-9A-Za-z]
//! <subfield-value>   ::= [^<us><rs>]*
//!
//! <sp> := #x20
//! <rs> := #x1e
//! <us> := #x1f
//! ```

mod borrowed;
pub mod parse;

pub use borrowed::{Field, Record, Subfield};
