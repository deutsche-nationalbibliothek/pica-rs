# split

Das `split`-Kommando teilt eine Menge an Datensätzen in mehrere Dateien
mit einer maximalen Anzahl pro Datei auf.

## Beschreibung

Mithilfe des `split`-Kommandos können alle Datensätze aus der Eingabe in
mehrere Dateien aufgeteilt werden, wobei jede Datei eine maximale Anzahl
an Datensätzen nicht überschreitet. Ein Anwendungsfall für das Splitting
könnte eine automatisierte [Stapelverarbeitung] oder die parallele
Verarbeitung der entstandenen Dateien sein.

Der folgende Aufruf des `split`-Kommandos teilt die zwölf Datensätze der
Eingabe (`DUMP.dat.gz`) in drei Dateien mit maximal fünf Datensätzen pro
Datei. Die ersten beiden Dateien (`0.dat` und `1.dat`) enthalten die
maximale Anzahl an Datensätzen, die letzte Datei (`2.dat`) die
restlichen zwei Datensätze.

```console
$ pica split -s 5 DUMP.dat.gz
$ tree
.
├── 0.dat
├── 1.dat
├── 2.dat
└── DUMP.dat.gz
```


## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-g`, `--gzip` — Komprimieren der Ausgabe im [Gzip]-Format.
* `--template` — Template für die Dateinamen. Der Platzhalter `{}` wird
  durch eine fortlaufende Nummer ersetzt.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt.
* `-o`, `--outdir` — Angabe, in welches Verzeichnis die Ausgabe
  geschrieben werden soll. Standardmäßig wird das aktuelle Verzeichnis
  verwendet.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Einige Kommandozeilen-Optionen lassen sich per Konfigurationsdatei
(`Pica.toml`) einstellen:

```toml
[split]
template = "CHUNK_{}.dat"
skip-invalid = true
gzip = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.


## Beispiele

### Benutzerdefinierte Dateinamen

Standardmäßig werden die erstellten Dateien beginnend bei `0`
durchnummeriert. Der Dateiname kann individuell mit der
`--template`-Option angepasst werden. Jedes Vorkommen der Zeichenfolge
`{}` im Template wird durch die aktuelle Nummer ersetzt. Endet die Datei
auf der Dateiendung `.gz`, wird die Ausgabe automatisch im [Gzip]-Format
komprimiert.

```console
$ pica split -s --template "CHUNK_{}.dat.gz" 10 DUMP.dat.gz
$ tree
.
├── CHUNK_0.dat.gz
├── CHUNK_1.dat.gz
└── DUMP.dat.gz

$ pica count --records CHUNK_0.dat.gz
10

$ pica count --records CHUNK_1.dat.gz
2

```

### Komprimierte Ausgabe

Mittels der Option `-g`/`--gzip` erfolgt eine Komprimierung der Ausgabe:

```console
$ pica split -s --gzip 10 DUMP.dat.gz
$ tree
.
├── 0.dat.gz
├── 1.dat.gz
└── DUMP.dat.gz

$ pica count --records 0.dat.gz
10

$ pica count --records 1.dat.gz
2

```

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

[Stapelverarbeitung]: https://de.wikipedia.org/wiki/Stapelverarbeitung
[Gzip]: https://de.wikipedia.org/wiki/Gzip
