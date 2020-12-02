//! This module provides functions to parse bibliographic records encoded in
//! PICA+ and parsers used in the cli commands. PICA+ is the internal format
//! used by the OCLC library system.
//!
//! NOTE: The code to parse excaped strings is based on the nom example; see
//! https://git.io/JkoOn.
//!
//! # PICA+ Grammar
//!
//! ```text
//! Record     ::= Field*
//! Field      ::= Tag Occurrence? Subfield* #x1e
//! Tag        ::= [012] [0-9]{2} [A-Z@]
//! Occurrence ::= '/' [0-9]{2,3}
//! Subfield   ::= Code Value
//! Code       ::= [a-zA-Z0-9]
//! Value      ::= [^#x1e#x1f]
//! ```
//!
//! [EBNF]: https://www.w3.org/TR/REC-xml/#sec-notation
