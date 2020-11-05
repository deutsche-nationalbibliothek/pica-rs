# pica-rs

[![CI](https://github.com/niko2342/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/niko2342/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![DOCs master](https://img.shields.io/badge/doc-master-orange.svg)](https://niko2342.github.io/pica-rs/pica/index.html)
[![Coverage Status](https://coveralls.io/repos/github/niko2342/pica-rs/badge.svg?branch=main)](https://coveralls.io/github/niko2342/pica-rs?branch=main)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Tools to work with bibliographic records encoded in Pica+.

## Commands

* [print](https://github.com/niko2342/pica-rs/wiki/Commands#print) — print records in human readable format
* [filter](https://github.com/niko2342/pica-rs/wiki/Commands#filter) — filter records by query expressions

## Usage

```bash
$ cat DUMP.dat
    | pica filter -s "(003@.0 == 123456789X && 002@.0 == Tx1) || 002@.0 != Ty2"
    | pica print
```

## Parser

```rust
use pica::Record;

fn main() {
    let record = Record::from_str("003@ \u{1f}0123456789\u{1e}")
        .expect("Invalid Pica+ record.");

    println!("Record = {:?}", record);
}
```

## TODO

- [ ] `pica convert` convert Pica+ records to other formats (csv, json, ...)
- [ ] `pica lint` check Pica+ records against field specs


## Related Projects

- [PICA::Data](https://github.com/gbv/PICA-Data) -  Perl module to handle PICA+ data.
- [PICA::Record](https://github.com/gbv/PICA-Record) -  Perl module to handle PICA+ records (deprecated).
- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) - Catmandu modules for working with PICA+ data.
- [luapica](http://jakobvoss.de/luapica/) - Handle PICA+ data in Lua.



