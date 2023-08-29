# partition

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Mittels des `partition`-Kommandos lassen sich Datensätze anhand eines
Unterfelds in Partitionen einteilen.

## Beschreibung

Lassen sich Datensätze anhand von den Wertausprägungen in einem
Unterfeld gruppieren, ist es mitunter hilfreich die Gesamtmenge der
Datensätze in Partitionen aufzuteilen. Ist das Unterfeld, nach dem
partitioniert werden soll, wiederholbar, sind die erzeugten Partitionen
i.d.R. nicht disjunkt. Ein Datensatz der das Unterfeld nicht besitzt,
geht in keine Partition ein.

> **Hinweis:** Die Werte des Unterfelds ergeben den Dateinamen der
> Partition. Es kann vorkommen, dass die Werte Sonderzeichen enthalten,
> die nicht vom Betriebssystem in Dateinamen erlaubt sind.


Im folgenden Beispiel wird pro Entitätencode im Feld `004B.a` eine
Partition erstellt, die alle GND-Entitäten enthält, die diesem
Entitätencode zugeordnet sind.

```console
$ pica partition -s "004B.a" DUMP.dat.gz -o out
$ tree out/
out
├── gik.dat
├── piz.dat
├── saz.dat
└── wit.dat

```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-g`, `--gzip` — Komprimieren der Ausgabe im [Gzip]-Format.
* `-t`, `--template` — Template für die Dateinamen. Der Platzhalter `{}`
  wird durch den Namen der Partition ersetzt.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt.
* `-o`, `--outdir` — Angabe, in welches Verzeichnis die Partitionen
  geschrieben werden sollen. Standardmäßig wird das aktuelle Verzeichnis
  verwendet.

## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Einige Kommandozeilen-Optionen lassen sich per Konfigurationsdatei
(`Pica.toml`) einstellen:

```toml
[partition]
template = "PART_{}.dat"
skip-invalid = true
gzip = true
```

## Beispiele

### Eingrenzen der Partitionen

Sollen nicht alle Partitionen erstellt werden, kann die Anzahl der
möglichen Partition durch die Angabe eines Filterausdrucks eingegrenzt
werden:

```console
$ pica partition -s "004B{a | a in ['piz', 'saz']}" DUMP.dat.gz -o out
$ tree out/
out
├── piz.dat
└── saz.dat

```

### Benutzerdefinierte Dateinamen

Standardmäßig werden die erstellten Partitionen nach den Werten im
Unterfeld benannt. Der Dateiname kann individuell mit der
`-t`/`--template`-Option angepasst werden. Jedes Vorkommen der Zeichenfolge
`{}` im Template wird durch den Wert des Unterfelds ersetzt. Endet die
Datei auf der Dateiendung `.gz`, wird die Ausgabe automatisch im
[Gzip]-Format komprimiert.

```console
$ pica partition -s "004B.a" --template "code_{}.dat.gz" DUMP.dat.gz -o out
$ tree out/
out
├── code_gik.dat.gz
├── code_piz.dat.gz
├── code_saz.dat.gz
└── code_wit.dat.gz

```

### Komprimierte Ausgabe

Mittels der Option `-g`/`--gzip` erfolgt eine Komprimierung der Ausgabe:

```console
$ pica partition -s "004B.a" --gzip DUMP.dat.gz -o out
$ tree out/
out
├── gik.dat.gz
├── piz.dat.gz
├── saz.dat.gz
└── wit.dat.gz

```


[Gzip]: https://de.wikipedia.org/wiki/Gzip
