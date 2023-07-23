# frequency

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Mithilfe des Kommandos `frequency` lässt sich die Häufigkeitsverteilung
aller Werte eines Unterfelds bestimmen.

## Beschreibung

Das Kommando `frequency` wird dazu genutzt, um die Häufigkeiten der
Werte eines Unterfelds zu bestimmen. Ist das zu untersuchende Feld bzw.
Unterfeld wiederholbar, dann gehen alle Wertausprägungen eines
Datensatzes in die Häufigkeitsverteilung ein. Die Ausgabe erfolgt
standardmäßig im CSV-Format.

Im folgenden Beispiel wird die Häufigkeitsverteilung des Unterfelds
`002@.0` (Satzart) ermittelt:

```console
$ pica frequency -s "002@.0" DUMP.dat.gz
Tu1,6
Tsz,2
Tg1,1
Tp1,1
Tpz,1
Ts1,1

```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-i`, `--ignore-case` — Groß- und Kleinschreibung wird bei Vergleichen
  ignoriert.
* `--strsim-threshold <value>` — Festlegen des Schwellenwerts beim
  Ähnlichkeitsvergleich von Zeichenketten mittels `=*`.
* `--reverse` — Ergebnisse werden in aufsteigender Reihenfolge
  ausgegeben.
* `-l`, `--limit` `<n>` — Eingrenzung der Ausgabe auf die häufigsten _n_
  Unterfeldwerte.
* `--threshold` `<n>` — Zeilen mit einer Häufigkeit < `<n>` ignorieren.
* `-H`, `--header` `<header>` — Kopfzeile, die den Ergebnissen
  vorangestellt wird.
* `-t`, `--tsv` — Ausgabe erfolgt im TSV-Format.
* `--translit` `<nf>` — Ausgabe wird in die angegebene Normalform
  transliteriert. Mögliche Werte: `nfd`, `nfkd`, `nfc` und `nfkc`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.

## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Die Option zum Ignorieren invalider Datensätze lässt sich in der
`Pica.toml` konfigurieren:

```toml
[frequency]
skip-invalid = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.


## Beispiele

### Hinzufügen einer Kopfzeile

Für die Dokumentation sowie die Verwendung in anderen Programmiersprachen
ist es häufig sinnvoll, eine Kopfzeile hinzuzufügen. Dies erfolgt mit der
Option `--header` bzw. `-H`. Die Namen der Spalten werden komma-separiert
angegeben. Eine Angabe von mehr als zwei Spalten ist nicht erlaubt.

```console
$ pica frequency -s --header "satzart,anzahl" "002@.0" DUMP.dat.gz
satzart,anzahl
Tu1,6
Tsz,2
Tg1,1
Tp1,1
Tpz,1
Ts1,1

```

### Eingrenzung auf bestimmte Felder

Oftmals sollen nicht alle Felder in die Berechnung der Häufigkeiten mit
einbezogen werden. Dies ist bspw. dann der Fall, wenn sich Felder anhand
eines Unterfelds unterschieden lassen, wie etwa durch die Angabe der
Metadatenherkunft. Durch Verwenden eines Pfad-Ausdrucks in {}-Notation
können nur die Felder ausgewählt werden, die einem bestimmten Kriterium
entsprechen.

Das folgende Beispiel bezieht nur die Felder `041R` in die Auswertung
mit ein, bei denen ein Unterfeld `4` existiert, das entweder `berc` oder
`beru` ist; Felder die diesem Kriterium nicht entsprechen, werden
ignoriert.

```console
$ pica frequency -s "041R{ 9 | 4 in ['berc', 'beru'] }" DUMP.dat.gz
040533093,2
040250989,1
040252434,1
040290506,1
...
```

### Eingrenzen der Treffermenge

Soll die Ergebnismenge auf die häufigsten _n_ Unterfeldwerte
eingeschränkt werden, wird dies mit der Option `--limit` bzw. `-l`
erreicht. Das nachfolgende Beispeil ermittelt die 3 häufigsten Werte im
Feld `041R.4`.

```console
$ pica frequency -s --limit 3 "041R.4" DUMP.dat.gz
beru,12
obal,5
vbal,4

```

### Eingrenzen der Treffermenge (Schwellenwert)

Die Treffermenge kann auch mittels der Angabe eines Schwellenwerts
eingeschänkt werden. Sollen nur die Werte angezeigt werden, die ab einem
Schwellenwert vorkommen, dann kann dies mit der Option
`--threshold`/`-t` erzielt werden:

```console
$ pica frequency -s --threshold 4 "041R.4" DUMP.dat.gz
beru,12
obal,5
vbal,4

```

### Änderung der Sortierreihenfolge (Limit)

Standardmäßig wird die Häufigkeitsverteilung absteigend ausgegeben,
d.h., der häufigste Wert steht in der Ausgabe oben[^fn1]. Soll das
Verhalten so geändert werden, dass die Ausgabe aufsteigend sortiert wird,
kann dies mit der Option `--reverse` bzw. `-r` erfolgen. Das folgende
Kommando sucht nach den vier Satzarten, die am wenigsten vorkommen:

```console
$ pica frequency -s -l 4 --reverse "002@.0" DUMP.dat.gz
Tg1,1
Tp1,1
Tpz,1
Ts1,1

```

### Ausgabe im TSV-Format

Die Ausgabe lässt sich mittels der Option `--tsv` (bzw. `-t`) in das
TSV-Format ändern.

```bash
$ pica frequency -s -l3 --tsv tests/data/dump.dat.gz
Tu1	6
Tsz	2
...
```

[^fn1]: Alle Werte mit gleicher Häufigkeit werden immer in lexikographisch
    aufsteigender Reihenfolge sortiert. Dies erfolgt unabhängig vom
    Parameter `--reverse`.
