# pica-rs

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![dependency status](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs/status.svg)](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs)

Das Toolkit _pica-rs_ ermöglicht eine effiziente Verarbeitung von
bibliografischen Metadaten, die in PICA+, dem internen Format des
[_OCLC_]-Katalogsystems, kodiert sind. Mit Hilfe verschiedener
[_Kommandos_] können aus den Metadaten elementare statistische Größen
ermittelt und aufbereitet werden. Zudem kann das Toolkit als
Brückentechnologie fungieren, um Metadaten für populäre Frameworks wie
[_Pandas_] oder [_Polars_] (Python), in Programmiersprachen wie [_R_]
oder für Excel nutzbar zu machen. Darüber hinaus eignet sich das Toolkit
für die Automatisierung von Metadaten-Workflows.

## Verwandte Projekte

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) — Catmandu modules for working with PICA+ data
- [Metafacture](https://github.com/metafacture) — Tool suite for metadata processing

[_Kommandos_]: referenz/kommandos/index.md
[_OCLC_]: https://www.oclc.org/de/
[_Pandas_]: https://pandas.pydata.org/
[_Polars_]: https://www.pola.rs/
[_R_]: https://www.r-project.org/
