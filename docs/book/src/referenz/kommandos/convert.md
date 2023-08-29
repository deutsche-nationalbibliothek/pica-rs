# `convert`

![stability-badge](https://img.shields.io/badge/stability-unstable-red?style=flat-square)

Das `convert`-Kommando wird für die Konvertierung von und in andere
Datenformate verwendet.

> **Hinweis:** Das `convert`-Kommando befindet sich in der aktiven
> Entwicklung. Funktionalitäten können unvollständig oder fehlerhaft
> sein. Änderungen am _command-line interface_ (CLI) sind nicht
> ausgeschlossen.

## Beschreibung

Das PICA-Format kann in verschiedene Datenformate serialisiert werden.
Das `convert`-Kommando ermöglicht es, Datensätze von einem Format in ein
anderes Format zu konvertieren. Es bietet insbesondere die Möglichkeit,
Datensätze, die nicht in normalisiertem PICA+ vorliegen, nach PICA+ zu
konvertieren, um sie durch andere [_Kommandos_] verarbeiten zu können.

Folgende Formate werden unterstützt:

* normalisiertes PICA+ (`plus`),
* binäres PICA (`binary`),
* PICA-Importformat (`import`),
* PICA-Plain (`plain`),
* PICA-JSON (`json`),
* und PICA-XML (`xml`).

Die Angabe der Datenformate erfolgt über die Optionen `--from` und
`--to`:

```bash
$ pica convert --from plus --to binary DUMP.dat.gz -o dump.bin
$ pica convert --from plus --to json DUMP.dat.gz -o dump.json
$ pica convert --from plus --to plain DUMP.dat.gz -o dump.plain
$ pica convert --from plus --to plus DUMP.dat.gz -o dump.dat
$ pica convert --from plus --to xml DUMP.dat.gz -o dump.xml
```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die nicht
  dekodiert werden konnten.
* `-f`, `--from` — Auswahl des Datenformats der Eingabe.
* `-t`, `--to` — Auswahl des Datenformats der Ausgabe.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.


[_Kommandos_]: ./index.md

