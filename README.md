# pica-rs

[![CI](https://github.com/niko2342/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/niko2342/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![DOCs master](https://img.shields.io/badge/doc-master-orange.svg)](https://niko2342.github.io/pica-rs/pica/index.html)
[![Coverage Status](https://coveralls.io/repos/github/niko2342/pica-rs/badge.svg?branch=main)](https://coveralls.io/github/niko2342/pica-rs?branch=main)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## About

This repository provides a collection of tools to work with bibliographic
records encoded in Pica+, the internal format of the OCLC Cataloging
system. The development of this tool was motivated by the wish to have a fast
and efficient way to transform Pica+ records to a data format, which can be
easily processed Python's [pandas](https://git.io/v7Qt8) library.

Most of the commands are inspired by the [xsv](https://git.io/JIoJG) toolkit.

## Installation

In order to install the pica tools a [Rust](https://www.rust-lang.org/) installation is required.
Just follow the [installation guide](https://www.rust-lang.org/learn/get-started) to get the Rust
programming language with the `cargo` package manager.

To install the latest stable release:

```bash
$ cargo install --git https://github.com/niko2342/pica-rs.git --branch main
```

## Commands

* [cat](https://github.com/niko2342/pica-rs/wiki/Commands#cat) — concatenate records from multiple files
* [completion](https://github.com/niko2342/pica-rs/wiki/Commands#completion) — generate a completions file for bash, fish or zsh.
* [filter](https://github.com/niko2342/pica-rs/wiki/Commands#filter) — filter records by query expressions
* [frequency](https://github.com/niko2342/pica-rs/wiki/Commands#frequency) — compute a frequency table of a subfield
* [invalid](https://github.com/niko2342/pica-rs/wiki/Commands#invalid) — filter out invalid records
* [json](https://github.com/niko2342/pica-rs/wiki/Commands#json) — serialize pica records to JSON
* [partition](https://github.com/niko2342/pica-rs/wiki/Commands#partition) — partition a list of records based on subfield values
* [print](https://github.com/niko2342/pica-rs/wiki/Commands#print) — print records in human readable format
* [sample](https://github.com/niko2342/pica-rs/wiki/Commands#sample) — selects a random permutation of records
* [select](https://github.com/niko2342/pica-rs/wiki/Commands#select) — write subfields to a CSV file
* [split](https://github.com/niko2342/pica-rs/wiki/Commands#split) — split a list of records into chunks

## Usage

A stream of records can be filtered by query expressions. Query expressions can
be simple comparisons (`==` (equal), `!=` (not equal) or `=~` (regex)), which
can be comninded by bollean connectives (`&&`, `||`), grouped in parentheses
(`(A || (B && C))`) and negated (`!(A || B)`).

```bash
$ pica filter -s "(003@.0 == '123456789X' || 002@{0 =~ '^Tp[123]$' || 0 == 'Ts1'}"
```

```bash
$ pica cat --skip-invalid DUMP1.dat.gz DUMP2.dat \
    | pica filter "(003@.0 == '123456789X' && 002@{0 == 'Ts1' || 0 == 'Ts2'}) || 002@.0 =~ '^Tp[123]$'" \
    | pica sample 42 \
    | pica print
```

## Parser

```rust
use pica::Record;

fn main() {
    let record = Record::decode("003@ \u{1f}0123456789\u{1e}")
        .expect("Invalid Pica+ record.");

    println!("Record = {:?}", record);
}
```

## Related Projects

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) - Catmandu modules for working with PICA+ data.
- [Metafacture](https://github.com/metafacture) - Tool suite for metadata processing.
- [PICA::Data](https://github.com/gbv/PICA-Data) -  Perl module to handle PICA+ data.
- [PICA::Record](https://github.com/gbv/PICA-Record) -  Perl module to handle PICA+ records (deprecated).
- [luapica](http://jakobvoss.de/luapica/) - Handle PICA+ data in Lua.

