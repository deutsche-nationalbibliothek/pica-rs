# slice

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Das `slice`-Kommando schneidet einen Teilbereich aus der Eingabe aus.


## Beschreibung

Mittels des `slice`-Kommandos kann ein Teilbereich aus der Eingabe
ausgeschnitten werden. Dabei wird der Teilbereich als halb-offenes
Intervall angegeben, wobei die Positionen von 0 gezählt werden.
Enthält bspw. die Eingabe 1.000 Zeilen, mit 990 Datensätzen und 10
ungültigen Zeilen, dann sind die Positionen von `0` bis 999
durchnummeriert.

Das folgende Beispiel extrahiert alle (gültigen) Datensätze aus den
Positionen 2 bis 4:


```console
$ pica slice -s --start 2 --end 5 DUMP.dat.gz -o slice.dat
$ pica count --records slice.dat
3

```


## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `--start` `<n>` — Startposition des Teilbereichs (Voreinstellung:
  `0`).
* `--end` `<n>` — Endposition des Teilbereichs; diese Position ist nicht
  mehr Teil der Ausgabe. Ist keine Endposition angegeben, wird der
  Teilbereich bis zum Ende der Eingabe fortgeführt. Diese Option ist
  nicht kombinierbar mit `--length`.
* `--length` `<n>` — Festlegen der maximalen Anzahl an Datensätzen, die
  in der Ausgabe enthalten sind. Diese Option kann nicht mit `--end`
  kombiniert werden.
* `-g`, `--gzip` — Komprimieren der Ausgabe im [Gzip]-Format.
* `--append` — Wenn die Ausgabedatei bereits existiert, wird die
  Ausgabe an die Datei angehangen. Ist das Flag nicht gesetzt, wird eine
  bestehende Datei standardmäßig überschrieben.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Einige Kommandozeilen-Optionen lassen sich per Konfigurationsdatei
(`Pica.toml`) einstellen:

```toml
[slice]
skip-invalid = true
gzip = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.


## Beispiele

### Ausschneiden eines Teilbereichs fester Größ

Wenn die Eingabe ausreichend Datensätze enthält, kann beginnend bei
einer festen Position (`--start`) eine Teilbereich mit einer festen
Länge (`--length`) ausgeschnitten werden.

Im folgenden Beispiel wird beginnend beim dritten Datensatz (Position 2)
ein Teilbereich mit einer Länge von zwei ausgeschnitten:

```console
$ pica slice -s --start 2 --length 2 DUMP.dat.gz -o slice.dat
$ pica count --records slice.dat
2

```


[Gzip]: https://de.wikipedia.org/wiki/Gzip
