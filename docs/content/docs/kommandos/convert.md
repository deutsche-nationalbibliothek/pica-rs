# convert

Das `convert`-Kommando wird für die Konvertierung von und in andere
Datenformate verwendet.

{{< hint danger >}}
**Hinweis:**
Das `convert`-Kommando befindet sich in der aktiven Entwicklung.
Funktionalitäten können unvollständig oder fehlerhaft sein. Änderungen
am _command-line interface_ (CLI) sind nicht ausgeschlossen.
{{< /hint >}}

## Beschreibung

Das PICA-Format kann in verschiedene Datenformate serialisiert werden.
Das `convert`-Kommando ermöglicht es, Datensätze von einem Format in ein
anderes Format zu konvertieren. Es bietet insbesondere die Möglichkeit,
Datensätze, die nicht in normalisiertem PICA+ vorliegen, nach PICA+ zu
konvertieren, um sie durch andere Kommandos verarbeiten zu können.

Folgende Formate werden unterstützt:

* normalisiertes PICA+ (`plus`),
* binäres PICA (`binary`),
* PICA-Importformat (`import`),
* PICA-Plain (`plain`),
* PICA-JSON (`json`),
* und PICA-XML (`xml`).

Die Angabe der Datenformate erfolgt über die Optionen `--from`/`-f` und
`--to`/`-t`:

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


[cat]: {{< relref "cat.md" >}}
[completions]: {{< relref "completions.md" >}}
[convert]: {{< relref "convert.md" >}}
[count]: {{< relref "count.md" >}}
[explode]: {{< relref "explode.md" >}}
[filter]: {{< relref "filter.md" >}}
[frequency]: {{< relref "frequency.md" >}}
[hash]: {{< relref "hash.md" >}}
[invalid]: {{< relref "invalid.md" >}}
[partition]: {{< relref "partition.md" >}}
[print]: {{< relref "print.md" >}}
[slice]: {{< relref "slice.md" >}}
[split]: {{< relref "split.md" >}}

[Gzip]: https://de.wikipedia.org/wiki/Gzip
