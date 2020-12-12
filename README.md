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

* [cat](https://git.io/JI6H2) — concatenate records from multiple files
* [completion](https://github.com/niko2342/pica-rs/wiki/Commands#completion) — generate a completions file for bash, fish or zsh.
* [filter](https://git.io/JI6HE) — filter records by query expressions
* [frequency](https://github.com/niko2342/pica-rs/wiki/Commands#frequency) — compute a frequency table of a subfield
* [invalid](https://github.com/niko2342/pica-rs/wiki/Commands#invalid) — filter out invalid records
* [json](https://github.com/niko2342/pica-rs/wiki/Commands#json) — serialize pica records to JSON
* [partition](https://github.com/niko2342/pica-rs/wiki/Commands#partition) — partition a list of records based on subfield values
* [print](https://github.com/niko2342/pica-rs/wiki/Commands#print) — print records in human readable format
* [sample](https://github.com/niko2342/pica-rs/wiki/Commands#sample) — selects a random permutation of records
* [select](https://github.com/niko2342/pica-rs/wiki/Commands#select) — write subfields to a CSV file
* [split](https://github.com/niko2342/pica-rs/wiki/Commands#split) — split a list of records into chunks

## Usage

### Cat

Multiple pica dumps can be concatenated to a single record stream:

```bash
$ pica cat -s -o DUMP12.dat DUMP1.dat DUMP2.dat.gz
```

### Filter

The key component of the tool is the ability to filter for records, which meet
a filter criterion. The basic building block are field expression, which
consists of an field tag (ex. `003@`) and optional occurrence (ex. `/00`) and a
subfield filter. These expressions can be combined to complex expressions by
the boolean connectives AND (`&&`) and OR (`||`). Boolean expressions are
evaluated lazy from left to right.

Simple subfield filter consists of the subfield code (single alpha-numerical
character, ex `0`) a comparison operator (equal `==`, not equal `!=` (not
equal) or regex `=~`) and a value enclosed in single quotes.. These simple
subfield expressions can be grouped in parentheses and combinded with boolean
connectives (ex. `(0 == 'abc' || 0 == 'def')`). There is also a special
existence operator, to check if a given subfield exists (`a?`).

**Examples**

```bash
$ pica filter -s "002@.0 =~ '^O(?!lfo)$' && 010@{a == 'ger' || a == 'eng'}" DUMP.dat
$ pica filter -s "002@.0 =~ '^O.*' && 044H{9? && b == 'GND'}" DUMP.dat
$ pica filter -s "010@{a == 'ger' || a == 'eng'} DUMP.dat
$ pica filter -s "003@{0 == '123456789X'}" DUMP.dat
$ pica filter -s "003@.0 == '123456789X'" DUMP.dat
```

### Frequency

The `frequency` command computes a frequency table of a subfield. The result is
formatted as CSV (value,count). The following example builds the frequency
table of the field `010@.a` of a filtered set of records.

```bash
$ pica filter --skip-invalid "002@.0 =~ '^A.*'" DUMP.dat.gz \
    | pica frequency "010@.a"

ger,2888445
eng,347171
...
```

## Related Projects

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) - Catmandu modules for working with PICA+ data.
- [Metafacture](https://github.com/metafacture) - Tool suite for metadata processing.
- [PICA::Data](https://github.com/gbv/PICA-Data) -  Perl module to handle PICA+ data.
- [PICA::Record](https://github.com/gbv/PICA-Record) -  Perl module to handle PICA+ records (deprecated).
- [luapica](http://jakobvoss.de/luapica/) - Handle PICA+ data in Lua.

