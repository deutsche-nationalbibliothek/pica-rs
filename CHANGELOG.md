# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.25.0] - 2023-05-30

### Added

* #783 Add filter options (`count`)
* #793 Add `--keep`/`--discard` option (`explode`)
* #791 Add `--limit`/`-l` option (`explode`)
* #790 Add XOR operator

### Fixed

* #776 Fix glibc error (cross builds)
* #779 Fix use of legacy numeric constants
* #781 Fix progress bar update (`frequency`)
* #784 Fix explode on copy level
* #778 Fix broken links

### Changed

* #772 Change license to EUPL-1.2
* #764 Migrate from `mdbook` to `hugo`
* #773 Refactor CI and daily workflow
* #794 Stabilize `explode` command
* #788 Bump MSRV to `1.76.0`


## [0.24.0] - 2023-02-05

### Added

* #753 select: add `--limit` (`-l`) option
* #750 frequency: add filter options

### Fixed

* #755 Fix processing of tsv/csv filter-lists

### Changed

* #756 msrv: bump to Rust 1.74.1


## [0.23.0] - 2023-01-19

* #736 Add existential and universal quantifier

### Fixed

* #737 Fix deserialization of matcher and path expressions


## [0.22.0] - 2023-12-15

### Changed

* #734 Improve release flags
* #716 Strip symbols in release builds
* #725 Move `FilterList` into `pica-utils` crate
* #719 Cleanup `pica-matcher` API and tests
* #724 Cleanup `pica-select` API and tests
* #722 Cleanup `pica-path` API and tests
* #709 Remove `$`-notation in a field matcher expression
* #707 Migrate parser code to winnow

### Added

* #733 Allow subfield code ranges in subfield matcher
* #732 Add subfield wildcard in path expressions
* #731 Add `--unique` flag to `frequency` command
* #728 Simplify matcher composite (`MatcherBuilder`)
* #720 Restrict level of group expressions


## [0.21.0] - 2023-11-15

### Changed

* #698 Remove deprecated path syntax
* #701 Move crates into `crates` dir


## [0.20.0] - 2023-09-14

### Added

* #694 Add `explode` command
* #693 Add progress bar


## [0.19.0] - 2023-08-23

### Added

* #687 Allow multi-field queries in `frequency` command
* #684 Allow code ranges in path expressions

### Changed

* #686 Adapt select/frequency to new path struct
* #685 Deprecate outdated path syntax

### Fixed

* #688 Allow `Not` operands in `Or` expressions


## [0.18.0] - 2023-07-27

### Added

* #637 Stabilize `print` command
* #641 Stabilize `sample` command
* #642 Add `--squash` and `--merge` option
* #644 Add `!^` and `!$` operator
* #658 Add unique-strategy config option (`cat` command)
* #672 Stabilize `select` command
* #673 Add contains relation matcher (`=?`)
* #674 Change `--threshold` behavior (`frequency` command)

### Changed

* #643 Print more helpful error message on `ParsePicaError`
* #653 Don't require filter argument when an expression file is given
* #654 Change `expr_file` short option from `-f` to `-F`

### Removed

* #639 Remove `xml` command
* #640 Remove `json` command


## [0.17.0] - 2023-06-30

### Added

* #622 Support boolean connectives in `select` command
* #624 Support allow- and deny-lists in `select` command
* #627 Add `hash` command

## [0.16.0] - 2023-05-26

### Added

* #612 Support of double quoted string literals
* #611 Allow negation of a field matcher in curly bracket notation
* #610 Add `convert` command

### Removed

* #613 Remove `--reduce` option

## [0.15.1] - 2023-03-31

### Fixed

* #605 Fix false positives of `!~` operator

## [0.15.0] - 2023-03-23

### Added

* #564 Add `--seed` option to `sample` command
* #592 Transliteration of matcher expressions

### Changed

* #590 Stabilize `slice` command
* #594 Stabilize `split` command
* #595 Stabilize `filter` command
* #598 Stabilize `partition` command
* #601 Deprecate `--reduce` option


## [0.14.1] - 2023-01-17

### Fixed

* #569 Properly handle records without an PPN (`filter`)


## [0.14.0] - 2023-01-16

### Fixed

* #563 Fix false positives of `not in` operator

### Added

* #557 Add short variant for reduce option (`filter`)
* #534 Add `cat` snapshot tests
* #524 Add `invalid` snapshot tests
* #525 Add long help (`invalid`)

### Changed

* #562 Use `Reader` and `ReaderBuilder` instead of `BufReadExt`
* #532 Improve performance of `cat` command
* #522 Use `BufReadExt` to process records (`invalid`)
* #523 Move reader/writer logic to config (`invalid`)
* #537 Stabilize `cat` command
* #538 Stabilize `completions` command
* #554 Stabilize `count` command
* #566 Cleanup release workflow
