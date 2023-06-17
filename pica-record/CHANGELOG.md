# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Added

* #625 Implement `Hash` for `ByteRecord`
* #626 Implement `sha256` for `ByteRecord`

## v0.1.0

### Added

* #521 Add `RecordRef::write_to` function
* #520 Add `BufReadExt` extension trait
* #528 Add `ByteRecord` writer API
* #530 Add `raw_data` field (`ByteRecord`)
* #562 Add `Reader` and `ReaderBuilder`

### Fixed

* #531 Fix wrong assert expression (`WriterBuilder`)

## 0.1.0 - 2022-10-14

### Added

* #489 Add `ByteRecord` and `StringRecord`
* #485 Add `Field` and `FieldRef`
* #484 Add `OccurenceRef` and `OccurenceRef`
* #487 Add `RecordRef` and `Record`
* #481 Add `SubfieldRef` and `Subfield`
* #483 Add `TagRef` and `Tag`
