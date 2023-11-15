<p align="center"><img  height="250" width="250" src="./.github/pica-rs_logo.png"></p>

<div align="center" markdown="1">

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/workflows/CI/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![dependency status](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs/status.svg)](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

</div>

<hr />

Das Toolkit _pica-rs_ ermöglicht eine effiziente Verarbeitung von
bibliografischen Metadaten, die in PICA+, dem internen Format des
[OCLC]-Katalogsystems, kodiert sind. Mithilfe verschiedener [Kommandos]
können aus den Metadaten elementare statistische Größen ermittelt und
aufbereitet werden. Zudem kann das Toolkit als Brückentechnologie
fungieren, um Metadaten für populäre Frameworks wie [Pandas] oder
[Polars] (Python), in Programmiersprachen wie [R] oder für Excel nutzbar
zu machen.

Das Projekt ist eine Entwicklung des Referats _Automatische
Erschließungsverfahren; Netzpublikationen_ (AEN) der [Deutschen
Nationalbibliothek][DNB] (DNB). Es wird für die Erstellung von
Datenanalysen sowie für die Automatisierung von Metadaten-Workflows
(Datenmanagement) im Rahmen der [automatischen Inhaltserschließung][AE]
genutzt. Weiterhin wird es zur Unterstützung der Forschungsarbeiten im
Projekt [Automatisches Erschließungssystem][KI] und für diverse andere
Datenanalysen in der DNB eingesetzt.

Die Mitwirkung an _pica-rs_ ist sehr erwünscht. Wir würden Sie bitten,
mögliche Fehler, Fragen und neue Ideen als [GitHub-Issues][Issues]
anzulegen. Diese werden wir dann intern beraten und möglichst zeitnah
ein Feedback geben.

Die [Installation] des Toolkits und alle Kommandos sind in der
[Dokumentation] beschrieben.

### Kommandos

* [cat] — Zusammenfügen (Konkatenieren) von Datensätzen
* [completions] — Erzeugung von Shell-Skripten zur Autovervollständigung
* [convert] — Konvertierung zwischen verschiedenen PICA-Formaten
* [count] — Zählen von Datensätzen, Feldern und Unterfeldern
* [explode] — Teilt Datensätze in Lokal- oder Exemplardatensätze auf
* [filter] — Filtert Datensätze anhand eines Kriteriums
* [frequency] — Ermitteln einer Häufigkeitsverteilung über ein oder
  mehrere Unterfelder
* [hash] — Erzeugt SHA-256-Hashwerte von Datensätzen
* [invalid] — Findet ungültige Zeilen in der Eingabe
* [partition] — Partitioniert Datensätze anhand eines Unterfelds
* [print] — Gibt Datensätze in einer menschenlesbaren Form aus
* sample — Wählt eine Zufallsstichprobe eines bestimmten Umfangs aus
* select — Selektiert ein oder mehrere Unterfelder und gibt die Daten im
  CSV- bzw. TSV-Format aus
* [slice] — Ausschneiden eines zusammenhängenden Teilbereichs aus der
  Eingabe
* [split] — Teilt eine Menge an Datensätzen in Dateien fester Größe


### Verwandte Projekte

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) — Catmandu modules for working with PICA+ data
- [Metafacture](https://github.com/metafacture) — Tool suite for metadata processing


[AE]: https://blog.dnb.de/erschliessungsmaschine-gestartet/
[DNB]: https://www.dnb.de/
[Dokumentation]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/
[Installation]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/anleitungen/installation.html
[Issues]: https://github.com/deutsche-nationalbibliothek/pica-rs/issues
[KI]: https://www.dnb.de/DE/Professionell/ProjekteKooperationen/Projekte/KI/ki_node.html
[Kommandos]: #kommandos
[OCLC]: https://www.oclc.org/
[Pandas]: https://pandas.pydata.org/
[Polars]: https://www.pola.rs/
[R]: https://www.r-project.org/

[cat]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/cat.html
[completions]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/completions.html
[convert]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/convert.html
[count]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/count.html
[explode]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/explode.html
[filter]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/filter.html
[frequency]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/frequency.html
[hash]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/hash.html
[invalid]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/invalid.html
[partition]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/partition.html
[print]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/print.html
[slice]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/slice.html
[split]: https://deutsche-nationalbibliothek.github.io/pica-rs/book/referenz/kommandos/split.html
