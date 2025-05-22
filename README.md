<p align="center"><img  height="250" width="250" src="./.github/pica-rs_logo.png"></p>

<div align="center" markdown="1">

[![CI](https://github.com/deutsche-nationalbibliothek/pica-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/deutsche-nationalbibliothek/pica-rs/actions/workflows/rust.yml)
[![Documentation](https://img.shields.io/badge/Documentation-main-orange.svg)](https://deutsche-nationalbibliothek.github.io/pica-rs/)
[![dependency status](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs/status.svg)](https://deps.rs/repo/github/deutsche-nationalbibliothek/pica-rs)

</div>

<hr />

Das Toolkit _pica-rs_ ermöglicht eine effiziente Verarbeitung von
bibliografischen Metadaten, die in PICA+, dem internen Format des
[OCLC]-Katalogsystems, kodiert sind. Mithilfe verschiedener [Kommandos]
können aus den Metadaten elementare statistische Größen ermittelt und
aufbereitet werden. Zudem kann das Toolkit als Brückentechnologie
fungieren, um Metadaten für populäre Frameworks wie [Polars] (Python), in
Programmiersprachen wie [R] oder für Excel nutzbar zu machen.

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
ein Feedback geben. Ferner kann das Forum [metadaten.community] zur
Diskussion genutzt werden.

Die Installation des Toolkits und alle Kommandos sind in der
[Dokumentation] beschrieben.

### Kommandos

* [check] — Überprüft Datensätze auf Konformität mit einem Regelwerk
* [concat] — Zusammenfügen (Konkatenieren) von Datensätzen
* [completions] — Erzeugung von Shell-Skripten zur Autovervollständigung
* [convert] — Konvertierung zwischen verschiedenen PICA-Formaten
* [count] — Zählen von Datensätzen, Feldern und Unterfeldern
* [describe] — Erstellt pro Feld eine Häufigkeitsverteilung der Unterfelder
* [explode] — Teilt Datensätze in Lokal- oder Exemplardatensätze auf
* [filter] — Filtert Datensätze anhand eines Kriteriums
* [frequency] — Ermitteln einer Häufigkeitsverteilung über ein oder
  mehrere Unterfelder
* [hash] — Erzeugt SHA-256-Hashwerte von Datensätzen
* [invalid] — Findet ungültige Zeilen in der Eingabe
* [partition] — Partitioniert Datensätze anhand eines Unterfelds
* [print] — Gibt Datensätze in einer menschenlesbaren Form aus
* [sample] — Wählt eine Zufallsstichprobe eines bestimmten Umfangs aus
* [select] — Selektiert ein oder mehrere Unterfelder und gibt die Daten im
  CSV- bzw. TSV-Format aus
* [slice] — Ausschneiden eines zusammenhängenden Teilbereichs aus der
  Eingabe
* [split] — Teilt eine Menge an Datensätzen in Dateien fester Größe


### Verwandte Projekte

- [Catmandu::Pica](https://metacpan.org/pod/Catmandu::PICA) — Catmandu modules for working with PICA+ data
- [Metafacture](https://github.com/metafacture) — Tool suite for metadata processing


[AE]: https://blog.dnb.de/erschliessungsmaschine-gestartet/
[DNB]: https://www.dnb.de/
[Dokumentation]: https://deutsche-nationalbibliothek.github.io/pica-rs/
[Issues]: https://github.com/deutsche-nationalbibliothek/pica-rs/issues
[KI]: https://www.dnb.de/ki-projekt
[metadaten.community]: https://metadaten.community/c/software-und-tools/pica-rs/31
[Kommandos]: #kommandos
[OCLC]: https://www.oclc.org/
[Polars]: https://www.pola.rs/
[R]: https://www.r-project.org/

[check]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/check.html
[concat]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/concat.html
[completions]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/completions.html
[convert]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/convert.html
[count]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/count.html
[describe]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/describe.html
[explode]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/explode.html
[filter]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/filter.html
[frequency]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/frequency.html
[hash]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/hash.html
[invalid]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/invalid.html
[partition]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/partition.html
[print]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/print.html
[sample]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/sample.html
[select]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/select.html
[slice]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/slice.html
[split]: https://deutsche-nationalbibliothek.github.io/pica-rs/commands/split.html

## Lizenz

Der Quellcode sowie die Releases sind lizenziert unter der [EUPL-1.2](./LICENSE).
