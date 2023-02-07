# pica-rs

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![Coverage Status](https://coveralls.io/repos/github/deutsche-nationalbibliothek/pica-rs/badge.svg?branch=main)](https://coveralls.io/github/deutsche-nationalbibliothek/pica-rs?branch=main)
[![dependency status](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs/status.svg)](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## About

This repository provides a collection of tools to work with bibliographic
records encoded in PICA+, the internal format of the OCLC Cataloging
system. The development of this tool was motivated by the wish to have a fast
and efficient way to transform PICA+ records to a data format, which can be
easily processed Python's [pandas](https://git.io/v7Qt8) library.

Most of the commands are inspired by the [xsv](https://git.io/JIoJG) toolkit.

## Installation

Binaries for Windows, Linux and macOS as well as `RPM` and `DEB` packages are available from [GitHub](https://github.com/deutsche-nationalbibliothek/pica-rs/releases).

In order to install the tools from source a [Rust](https://www.rust-lang.org/) installation is required. Just follow the [installation
guide](https://www.rust-lang.org/learn/get-started) to get the Rust programming language with the `cargo` package manager. To build
this project from source Rust 1.58.1 or newer is required.

To install the latest stable release:

```bash
$ cargo install --git https://github.com/deutsche-nationalbibliothek/pica-rs --tag v0.14.1 pica
```

## Commands

| Command                 | Stability | Desciption                                                        |
|-------------------------|-----------|-------------------------------------------------------------------|
| [cat](#cat)             | stable    | concatenate records from multiple files                           |
| completions             | stable    | generate a completions file for bash, fish or zsh                 |
| [count](#count)         | stable    | count records, fields and subfields                               |
| [filter](#filter)       | beta      | filter records by query expressions                               |
| [frequency](#frequency) | stable    | compute a frequency table of a subfield                           |
| [invalid](#invalid)     | stable    | write input lines, which can't be decoded as normalized PICA+     |
| [partition](#partition) | beta      | partition a list of records based on subfield values              |
| [print](#print)         | beta      | print records in human readable format                            |
| [sample](#sample)       | beta      | selects a random permutation of records                           |
| [select](#select)       | beta      | select subfield values from records                               |
| [slice](#slice)         | stable    | return records withing a range (half-open interval)               |
| [split](#split)         | stable    | split a list of records into chunks                               |
| [json](#json)           | beta      | serialize records in JSON                                         |
| [xml](#xml)             | unstable  | serialize records into [PICA XML](https://format.gbv.de/pica/xml) |

## Usage

PICA+ data is read from input file(s) or standard input in normalized PICA+
serialization. Compressed `.gz` archives are decompressed.

### Cat

Multiple pica dumps can be concatenated to a single stream of records:

```bash
$ pica cat -s -o DUMP12.dat DUMP1.dat DUMP2.dat.gz
```

### Count

To count the number of records, fields and subfields use the following command:

```bash
$ pica count -s dump.dat.gz
records 7
fields 247
subfields 549
```

### Filter

The key component of the tool is the ability to filter for records, which meet
an expression as filter criterion. The basic building block of these expressions
are *field expressions*, which consists of a *field tag* (e.g.
`003@`), an optional *occurrence* (e.g `/03`), and a *subfield filter*.

A simple field tag consists of level number (`0`, `1`, or `2`) followed by two
digits and a character (`A` to `Z` and `@`). The dot (`.`) can be used as
wildcard for any character and square brackets can be used for alternative
characters (e.g. `04[45].` matches all fields starting with `044` or `045` but
no occurrence).

Occurrence `/00` and no occurence are equivalent, `/*` matches all occurrences
(including zero) and `/01-10` matches any occurrences between `/01` and `/10`.

Simple subfield filter consists of the subfield code (single
alpha-numerical character, ex `0`) a comparison operator (equal `==`,
not equal `!=` not equal, starts with prefix `=^`, ends with suffix
`=$`, regex `=~`/`!~`, `in` and `not in`) and a value enclosed in
single quotes. These simple subfield expressions can be grouped in
parentheses and combined with boolean connectives (ex. `(0 == 'abc' || 0 == 'def')`).

A special existence operator can be used to check if a given field (`012A/00?`)
or a subfield (`002@.0?` or `002@$0?`) exists.  To test for the number of times
a field or subfield exists in a record or field respectively, use the
cardinality operator `#` with a comparison operator (e.g. `#010@ > 1` or
`010@{#a == 1 && a == 'ger'}`).

Field expressions can be combined to complex expressions by the boolean
connectives AND (`&&`) and OR (`||`). Boolean expressions can be grouped
with parenthesis. Precedence of AND is higher than OR, so `A || B && C` is
equivalent to `A || (B && C)`. Expressions are evaluated lazy from left to
right so given `A || B` if `A` is true than `B` will not be evaluated.

Option `--reduce` can be used to reduce records to a limited set of fields.

**Examples**

```bash
$ pica filter -s "002@.0 =~ '^O[^a].*$' && 010@{a == 'ger' || a == 'eng'}" DUMP.dat
$ pica filter -s "002@.0 =~ '^O.*' && 044H{9? && b == 'GND'}" DUMP.dat
$ pica filter -s "010@{a == 'ger' || a == 'eng'}" DUMP.dat
$ pica filter -s "041A/*.9 in ['123', '456']" DUMP.dat
$ pica filter -s "010@.a in ['ger', 'eng']" DUMP.dat
$ pica filter -s "010@.a not in ['ger', 'eng']" DUMP.dat
$ pica filter -s "003@{0 == '123456789X'}" DUMP.dat
$ pica filter -s "003@.0 == '123456789X'" DUMP.dat
$ pica filter -s "002@.0 =^ 'Oa'" DUMP.dat
$ pica filter -s "012[AB]/00?" --reduce "003@,012[AB]/00" DUMP.dat
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

### Invalid

Most commands support option `--skip-invalid` to skip invalid input lines,
which can't be decoded as normalized PICA+. The `invalid` command can be used
to extract these invalid lines only.

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

The `print` command is used to print records in humand-readable
[PICA Plain](http://format.gbv.de/pica/plain) format.

```bash
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica print
003@ $0123456789$ab
```

### Sample

The `sample` command selects a random permutation of records of the given
sample size. This command is particularly useful in combination with the
`filter` command for QA purposes.

The following command filters for records, that have a field `002@` with a
subfield `0` that is `Tp1` or `Tpz` and selects a random permutation of 100
records.

```bash
$ pica filter -s "002@.0 =~ '^Tp[1z]'" | pica sample 100 -o samples.dat
```

### Select

This command selects subfield values of a record and emits them in CSV format.
A select expression consists of a non-empty list of selectors. A selector
references a field and a list of subfields or an static value enclosed in
single quotes. If a selector's field or any subfield is repeatable, the rows
are "multiplied". For example, if the first selector returns one row, the
second selector two rows and a third selecor 3 rows, the result will contain
`1 * 2 * 3 = 6` rows. Non-existing fields or subfields results in empty columns.

```bash
$ pica select -s "003@.0,012A/*{a,b,c}" DUMP.dat.gz
123456789X,a,b,c
123456789X,d,e,f

$ pica select -s "003@.0, 'foo', 'bar'" DUMP.dat.gz
123456789X,foo,bar
123456789X,foo,bar
```

To filter for fields matching a subfield filter, the first part of a complex
field expression can be a filter. The following select statement takes only
`045E` fields into account, where the expression `E == 'm'` evaluates to
`true`.

```bash
$ pica select -s "003@.0, 045E{ E == 'm', e}
...
```

In order to use TAB-character as field delimiter add the `--tsv` option:

```bash
$ pica select -s --tsv "003@.0,012A{a,b,c}" DUMP.dat.gz
123456789X    a    b    c
123456789X    d    e    f
```

### Slice

The `slice` command returns records within a range. The lower bound is
inclusive, whereas the upper bound is exclusive (half-open interval).
The numbering of records is zero-based; the first record is at position
0, the second at position 1, and so on. A range specified by `--start`
and `--end` might contain less elements, if the range contains invalid
records.

See `pica slice --help` for a detailed description of all options.

Examples:

```bash
# get records at position 1, 2 or 3 (without invalid record)
$ pica slice --skip-invalid --start 1 --end 4 -o slice.dat DUMP.dat

# get 10 records from position 10
$ pica slice --skip-invalid --start 10 --length 10 -o slice.dat DUMP.dat
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

### JSON

This command serializes the internal representation of record to JSON:

```bash
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica json | jq .
[
  {
    "fields": [
      {
        "name": "003@",
        "subfields": [
          {
            "name": "0",
            "value": "123456789"
          },
          {
            "name": "a",
            "value": "b"
          }
        ]
      }
    ]
  }
]
```

The result can be processed with other tools and programming languages. To get [PICA JSON](http://format.gbv.de/pica/json) format you can pipe the result to this [jq](https://stedolan.github.io/jq/) command:

    jq -c '.[]|.fields|map([.tag,.occurrence]+(.subfields|map(.tag,.value)))'

### XML

The `xml` command converts records into the [PICA XML](https://format.gbv.de/pica/xml) format.
More information can be found in the [GBV Wiki](https://verbundwiki.gbv.de/display/VZG/PICA+XML+Version+1.0).

```
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica xml
<?xml version="1.0" encoding="utf-8"?>
<collection xmlns="info:srw/schema/5/picaXML-v1.0" xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="info:srw/schema/5/picaXML-v1.0">
  <record>
    <datafield tag="003@">
      <subfield code="0">123456789</subfield>
    </datafield>
  </record>
</collection>
```

## Related Projects

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) - Catmandu modules for working with PICA+ data
- [PICA::Data](https://github.com/gbv/PICA-Data) -  Perl module and command line tool to handle PICA+ data
- [Metafacture](https://github.com/metafacture) - Tool suite for metadata processing
- [pica-data-js](https://github.com/gbv/pica-data-js) - Handle PICA+ data in JavaScript
- [luapica](http://jakobvoss.de/luapica/) - Handle PICA+ data in Lua
- [picaplus](https://github.com/FID-Judaica/picaplus) - tooling for working with PICA+
- [PICA::Record](https://github.com/gbv/PICA-Record) -  Perl module to handle PICA+ records (deprecated)

## License

This project is dual-licensed under [MIT](LICENSE) or the [UNLICENSE](UNLICENSE).
