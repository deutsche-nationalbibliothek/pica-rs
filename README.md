# pica-rs

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![Coverage Status](https://coveralls.io/repos/github/deutsche-nationalbibliothek/pica-rs/badge.svg)](https://coveralls.io/github/deutsche-nationalbibliothek/pica-rs)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## About

This repository provides a collection of tools to work with bibliographic
records encoded in Pica+, the internal format of the OCLC Cataloging
system. The development of this tool was motivated by the wish to have a fast
and efficient way to transform Pica+ records to a data format, which can be
easily processed Python's [pandas](https://git.io/v7Qt8) library.

Most of the commands are inspired by the [xsv](https://git.io/JIoJG) toolkit.

## Installation

In order to install the pica tools a [Rust](https://www.rust-lang.org/)
installation is required.  Just follow the [installation
guide](https://www.rust-lang.org/learn/get-started) to get the Rust programming
language with the `cargo` package manager.

To install the latest stable release:

```bash
$ cargo install --git https://github.com/niko2342/pica-rs.git --branch main
```

## Commands

* [cat](https://git.io/JI6H2) — concatenate records from multiple files
* [completion](https://github.com/niko2342/pica-rs/wiki/Commands#completion) — generate a completions file for bash, fish or zsh
* [filter](https://git.io/JI6HE) — filter records by query expressions
* [frequency](https://git.io/JIiG7) — compute a frequency table of a subfield
* [invalid](https://github.com/niko2342/pica-rs/wiki/Commands#invalid) — filter out invalid records
* [json](https://git.io/JIiGy) — serialize pica records to JSON
* [partition](https://git.io/JIiZy) — partition a list of records based on subfield values
* [print](https://git.io/JIinZ) — print records in human readable format
* [sample](https://git.io/JIica) — selects a random permutation of records
* [select](https://git.io/JIiWZ) — write subfields to a CSV file
* [split](https://git.io/JIiWE) — split a list of records into chunks

## Usage

### Cat

Multiple pica dumps can be concatenated to a single stream of records:

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
character, ex `0`) a comparison operator (equal `==`, not equal `!=` not equal,
starts with prefix `=^`, ends with suffix `=$` or regex `=~`) and a value
enclosed in single quotes.. These simple subfield expressions can be grouped in
parentheses and combinded with boolean connectives (ex. `(0 == 'abc' || 0 ==
'def')`). There is also a special existence operator, to check if a given
subfield exists (`a?`).

**Examples**

```bash
$ pica filter -s "002@.0 =~ '^O(?!lfo)$' && 010@{a == 'ger' || a == 'eng'}" DUMP.dat
$ pica filter -s "002@.0 =~ '^O.*' && 044H{9? && b == 'GND'}" DUMP.dat
$ pica filter -s "010@{a == 'ger' || a == 'eng'} DUMP.dat
$ pica filter -s "041A/*.9 in ['123', '456']" DUMP.dat
$ pica filter -s "0100.a in ['ger', 'eng']" DUMP.dat
$ pica filter -s "003@{0 == '123456789X'}" DUMP.dat
$ pica filter -s "003@.0 == '123456789X'" DUMP.dat
$ pica filter -s "002@.0 =^ 'Oa'" DUMP.dat
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

### JSON

To serialize a record to JSON, just run the following command:

```bash
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica json | jq .
[
  [
    {
      "tag": "003@",
      "occurrence": null,
      "subfields": [
        {
          "code": "0",
          "value": "123456789"
        },
        {
          "code": "a",
          "value": "b"
        }
      ]
    }
  ]
]
```

### Partition

In order to split a list of records into chunks based on a subfield value use
the `partition` command. Note that if the subfield is repeatable, the record
will be written to all partitions.

```bash
$ pica partition -s -o outdir "002@.0" DUMP.dat.gz
$ tree outdir/
outdir
├── Aa.dat
├── Aal.dat
├── Aan.dat
├── ...
```

### Print

The `print` command is used to print records in a human-readable format. The
format is similar to PICA3.

```bash
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica print
003@ $0 123456789 $a b

```

### Sample

The `sample` command selects a random permutation of records of the given
sample size. This command is particularly useful in combination with the
`filter` command for QA purposes.

The following command filters for records, that have a fied `002@` with a
subfield `0` that is `Tp1` or `Tpz` and selects a random permutation of 100
records.

```bash
$ pica filter -s "002@.0 =~ '^Tp[1z]'" | pica sample 100 -o samples.dat
```

### Select

This command selects subfields of a record and print them as CSV data. A select
expression consists of a non-empty list of selectors. A selector references a
field and a list of subfields. If a selector's field or any subfield is
repeatable, the rows are "multiplied". For example, if the first selector
returns one row, the second selector two rows and a third selecor 3 rows, the
result will contain `1 * 2 * 3 = 6` rows. Non-existing fields or subfields
results in an empty column.

The list of the subfield values can be changed by range expressions. For
example, `a[0]` takes just the first element, `a[1..]` takes all values from
index 1, `a[1..3]` takes all values with an index from 1 to 2 (half-open
range), `a[1..=3]` takes all values with an index from 1 to 3 and `a[..]` takes
all values (default).

```bash
$ pica select -s "003@.0,012A{a[..],b[0],c}" DUMP.dat.gz
123456789X,a,b,c
123456789X,d,e,f
```

### Split

This command is used to split a list of records into chunks of a given
size. The default filename is `{}.dat`, whereby the curly braces are replaced
by the number of the chunk.

```
$ pica split --skip-invalid --outdir outdir --template "CHUNK_{}.dat" 100 DUMP.dat
$ tree outdir
outdir
├── CHUNK_0.dat
├── CHUNK_10.dat
├── ...
```

## Related Projects

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) - Catmandu modules for working with PICA+ data.
- [Metafacture](https://github.com/metafacture) - Tool suite for metadata processing.
- [PICA::Data](https://github.com/gbv/PICA-Data) -  Perl module to handle PICA+ data.
- [PICA::Record](https://github.com/gbv/PICA-Record) -  Perl module to handle PICA+ records (deprecated).
- [luapica](http://jakobvoss.de/luapica/) - Handle PICA+ data in Lua.
- [picaplus](https://github.com/FID-Judaica/picaplus)  tooling for working with pica+
