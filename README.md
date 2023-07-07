# pica-rs

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![Coverage Status](https://coveralls.io/repos/github/deutsche-nationalbibliothek/pica-rs/badge.svg?branch=main)](https://coveralls.io/github/deutsche-nationalbibliothek/pica-rs?branch=main)
[![dependency status](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs/status.svg)](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## About

This repository provides a collection of tools to work with
bibliographic records encoded in PICA+, the internal format of the OCLC
Cataloging system. The development of this tool was motivated by the
wish to have a fast and efficient way to transform PICA+ records to a
data format, which can be easily processed Python's
[pandas](https://git.io/v7Qt8) library.

Most of the commands are inspired by the [xsv](https://git.io/JIoJG)
toolkit.

## Installation

Binaries for Windows, Linux and macOS as well as `RPM` and `DEB`
packages are available from
[GitHub](https://github.com/deutsche-nationalbibliothek/pica-rs/releases).

In order to install the tools from source a
[Rust](https://www.rust-lang.org/) installation is required. Just follow
the [installation guide](https://www.rust-lang.org/learn/get-started) to
get the Rust programming language with the `cargo` package manager. To
build this project from source Rust 1.58.1 or newer is required.

To install the latest stable release:

```bash
$ cargo install --git https://github.com/deutsche-nationalbibliothek/pica-rs --tag v0.17.0 pica
```

## Commands

| Command                 | Stability  | Desciption                                                        |
|-------------------------|------------|-------------------------------------------------------------------|
| [cat](#cat)             | stable     | concatenate records from multiple files                           |
| completions             | stable     | generate a completions file for bash, fish or zsh                 |
| [convert](#convert)     | unstable   | convert PICA+ into other formats (Plain, JSON, XML, etc.)         |
| [count](#count)         | stable     | count records, fields and subfields                               |
| [filter](#filter)       | stable     | filter records by query expressions                               |
| [frequency](#frequency) | stable     | compute a frequency table of a subfield                           |
| [hash](#hash)           | unstable   | compute SHA-256 checksums of records                              |
| [invalid](#invalid)     | stable     | write input lines, which can't be decoded as normalized PICA+     |
| [partition](#partition) | stable     | partition a list of records based on subfield values              |
| [print](#print)         | stable     | print records in human readable format                            |
| [sample](#sample)       | stable     | selects a random permutation of records                           |
| [select](#select)       | beta       | select subfield values from records                               |
| [slice](#slice)         | stable     | return records withing a range (half-open interval)               |
| [split](#split)         | stable     | split a list of records into chunks                               |

## Usage

PICA+ data is read from input file(s) or standard input in normalized
PICA+ serialization. Compressed `.gz` archives are decompressed.

### Cat

Multiple pica dumps can be concatenated to a single stream of records:

```bash
$ pica cat -s -o DUMP12.dat DUMP1.dat DUMP2.dat.gz
```

### Convert

The `convert` command can be used to convert PICA+ to the following
formats:

* Binary PICA (`binary`)
* PICA Import format (`import`)
* PICA JSON (`json`)
* Humand-readable PICA+ (`plain`)
* PICA+ (`plus`)
* PICA-XML (`xml`)

> :warning: At the moment only PICA+ is supported as input format.
> Reading from other formats will be added later.

Examples:

```bash
$ pica convert --from plus --to binary DUMP.dat.gz -o dump.bin
$ pica convert --from plus --to json DUMP.dat.gz -o dump.json
$ pica convert --from plus --to plain DUMP.dat.gz -o dump.plain
$ pica convert --from plus --to plus DUMP.dat.gz -o dump.dat
$ pica convert --from plus --to xml DUMP.dat.gz -o dump.xml
```

### Count

To count the number of records, fields and subfields use the following
command:

```bash
$ pica count -s dump.dat.gz
records 7
fields 247
subfields 549
```

### Filter

The key component of the tool is the ability to filter for records,
which meet an expression as filter criterion. The basic building block
of these expressions are *field expressions*, which consists of a *field
tag* (e.g. `003@`), an optional *occurrence* (e.g `/03`), and a
*subfield filter*.

A simple field tag consists of level number (`0`, `1`, or `2`) followed
by two digits and a character (`A` to `Z` and `@`). The dot (`.`) can be
used as wildcard for any character and square brackets can be used for
alternative characters (e.g. `04[45].` matches all fields starting with
`044` or `045` but no occurrence).

Occurrence `/00` and no occurence are equivalent, `/*` matches all
occurrences (including zero) and `/01-10` matches any occurrences
between `/01` and `/10`.

Simple subfield filter consists of the subfield code (single
alpha-numerical character, ex `0`) a comparison operator (equal `==`,
not equal `!=` not equal, starts with prefix `=^`, starts not with
prefix `!^`, ends with suffix `=$`, regex `=~`/`!~`, `in` and `not in`)
and a value enclosed in single quotes. These simple subfield expressions
can be grouped in parentheses and combined with boolean connectives (ex.
`(0 == 'abc' || 0 == 'def')`).

A special existence operator can be used to check if a given field
(`012A/00?`) or a subfield (`002@.0?` or `002@$0?`) exists.  To test for
the number of times a field or subfield exists in a record or field
respectively, use the cardinality operator `#` with a comparison
operator (e.g. `#010@ > 1` or `010@{#a == 1 && a == 'ger'}`).

Field expressions can be combined to complex expressions by the boolean
connectives AND (`&&`) and OR (`||`). Boolean expressions can be grouped
with parenthesis. Precedence of AND is higher than OR, so `A || B && C` is
equivalent to `A || (B && C)`. Expressions are evaluated lazy from left to
right so given `A || B` if `A` is true than `B` will not be evaluated.

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
```

Options `--keep` (`-k`) and `--discard` (`-d`) can be used to reduce
records to a limited set of fields by a specified list of field- and
occurence matcher.

**Examples**

```bash
$ pica filter -s "012[AB]/00?" --keep "003@,012[AB]/00" DUMP.dat
$ pica filter -s "003@?" --discard "2.../*" DUMP.dat
```

### Frequency

The `frequency` command computes a frequency table of a subfield. The
result is formatted as CSV (value,count). The following example builds
the frequency table of the field `010@.a` of a filtered set of records.

```bash
$ pica filter -s "002@.0 =~ '^A.*'" DUMP.dat.gz | pica frequency "010@.a"
ger,2888445
eng,347171
...
```

An optional filter can be used if only a subset of fields should be
taken into account:

```bash
$ pica pica frequency "044H{ 9 | b == 'GND' && H == 'aepgnd' && 9? }"
0123456789,123
...
...
```

### Hash

The `hash` command computes SHA-256 checksums of records and writes the
IDN and hash values as CSV/TSV. The checksums can be used to track
new and changed records. The checkums is computed over the complete
PICA+ record including a newline separator.

```bash
$ pica hash tests/snapshot/data/algebra.dat tests/snapshot/data/math.dat.gz
idn,sha256
040011569,ca9add6db02315df1aeee941b8aced2f63968499594dcb0d88ba54df0181d428
040379442,7635e838185237014c6575009c184ecac2ac106420f543b148e0794723a71bab
```

### Invalid

Most commands support option `--skip-invalid` to skip invalid input
lines, which can't be decoded as normalized PICA+. The `invalid` command
can be used to extract these invalid lines only.

### Partition

In order to split a list of records into chunks based on subfield
values, use the `partition` command. Note that if the field and/or
subfield is repeatable, the record will be written to all partitions
(duplicate values will be removed), thus the resulting partitions may
not be disjoint. Records that don't have the field/subfield, won't be
written to a partition.

```bash
$ pica partition -s -o outdir "002@.0" DUMP.dat.gz
$ tree outdir/
outdir
├── Aa.dat
├── Aal.dat
├── Aan.dat
├── ...
```

The filename of each partition is based on the subfield value by
default. In order to change the filename, use the `--template` (`-t`)
option. Any occurence of the placeholder `{}` will be replaced by the
value:

```bash
$ pica partition -s --template "BBG_{}.dat" -o outdir "002@.0" DUMP.dat.gz
$ tree outdir/
outdir
├── BBG_Aa.dat
├── BBG_Aal.dat
├── BBG_Aan.dat
├── ...
```

### Print

The `print` command is used to print records in humand-readable format.
Multiple records are separated by newline.

```bash
$ echo -e "003@ \x1f0123456789\x1fab\x1e" | pica print
003@ $0 123456789 $a b
```

### Sample

The `sample` command selects a random permutation of records of the
given sample size. This command is particularly useful in combination
with the `filter` command for QA purposes.

The following command filters for records, that have a field `002@` with
a subfield `0` that is `Tp1` or `Tpz` and selects a random permutation
of 100 records.

```bash
$ pica filter -s "002@.0 =~ '^Tp[1z]'" | pica sample 100 -o samples.dat
```

### Select

This command selects subfield values of a record and emits them in CSV
format. A select expression consists of a non-empty list of selectors. A
selector references a field and a list of subfields or an static value
enclosed in single quotes. If a selector's field or any subfield is
repeatable, the rows are "multiplied". For example, if the first
selector returns one row, the second selector two rows and a third
selecor 3 rows, the result will contain `1 * 2 * 3 = 6` rows.
Non-existing fields or subfields results in empty columns.

```bash
$ pica select -s "003@.0,012A/*{a,b,c}" DUMP.dat.gz
123456789X,a,b,c
123456789X,d,e,f

$ pica select -s "003@.0, 'foo', 'bar'" DUMP.dat.gz
123456789X,foo,bar
123456789X,foo,bar
```

To filter for fields matching a subfield filter, the first part of a
complex field expression can be a filter. The following select statement
takes only `045E` fields into account, where the expression `E == 'm'`
evaluates to `true`.

```bash
$ pica select -s "003@.0, 045E{ (e, E) | E == 'm' }
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
size. The default filename is `{}.dat`, whereby the curly braces are
replaced by the number of the chunk.

```
$ pica split -s -o outdir --template "CHUNK_{}.dat" 100 DUMP.dat
$ tree outdir
outdir
├── CHUNK_0.dat
├── CHUNK_10.dat
├── ...
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
