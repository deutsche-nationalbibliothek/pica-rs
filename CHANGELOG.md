# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* #637 Stabilize `print` command
* #641 Stabilize `sample` command
* #642 Add `--squash` and `--merge` option
* #644 Add `!^` and `!$` operator
* #658 Add unique-strategy config option (`cat` command)
* #672 Stabilize `select` command

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
