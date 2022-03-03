# frequency

> Hinweis: Das Kommando befindet sich im Beta-Status und wird gerade intensiv getestet, bevor
> es als stabil freigegeben wird. Änderungen am _command-line interface_ (CLI) sowie das
> Auftreten kleinerer Fehler ist möglich.

Mithilfe des Kommandos `frequency` lässt sich die Häufigkeitsverteilung aller Wertausprägungen
eines Unterfelds bestimmen.

## Beschreibung

Das Kommando `frequency` wird dazu genutzt, um die Häufigkeiten der Wertausprägungen eines
Unterfelds zu bestimmen. Ist das zu untersuchende Feld bzw. Unterfeld wiederholbar, dann
gehen alle Wertausprägungen eines Datensatzes in die Häufigkeitsverteilung ein. Aus der
Ergebnisdatei lässt sich dann durch geeignete Tools eine grafische Darstellung (Histogramm)
oder die relative Häufigkeitsverteilung berechnen. Die Ausgabe erfolgt im CSV-Format.

Im folgenden Beispiel wird die Häufigkeitsverteilung des Unterfelds `010@.a` (Sprache des
Textes) ermittelt. Sowohl das Feld `010@` als auch das Unterfeld `a` sind wiederholbar und
somit werden Datensätze die sowohl einen Sprachencode `ger` als auch `eng` erhalten haben
für jede Wertausprägung gezählt.

```bash
$ pica frequency "010@.a" FILE.dat
ger,2888445
eng,347171
...
```

### Hinzufügen einer Kopfzeile

Für die Dokumentation sowie die Verwendung in anderen Programmiersprachen ist es häufig
sinnvoll eine Kopfzeile hinzuzufügen. Dies erfolgt mit der Option `--header` bsw. `-H`.
Die Namen der Spalten werden komma-separiert angegeben. Die Angabe von mehr als zwei
Spalten ist nicht erlaubt.

```bash
$ pica frequency --header "sprache,anzahl" "010@.a" A.dat
sprache,anzahl
ger,2888445
eng,347171
...
```

### Eingrenzen der Treffermenge (Limit)

Soll die Treffermenge auf die _n_-häufigsten Wertausprägungen eingeschränkt werden, kann
dies mit der Option `--limit` bzw. `-l` erfolgen. Das folgende Beispiel sucht nach den
fünf häufigsten Sprachencodes:

```bash
$ pica frequency --limit 5 --header "sprache,anzahl" "010@.a" A.dat
sprache,anzahl
ger,4944293
eng,829241
fre,140055
spa,61131
ita,60113
```

### Eingrenzen der Treffermenge (Schwellenwert)

Die Treffermenge kann auch mittels der Angabe eines Schwellenwerts eingeschänkt werden.
Sollen nur die Wertausprägungen angezeigt werden, die häufiger als ein Schwellenwert _n_
vorkommen, dann kann dies mit der Option `--threshold` bzw. `-t` erzielt werden:

```bash
$ pica frequency --theshold 100000 -H "sprache,anzahl" "010@.a" A.dat
sprache,anzahl
ger,4944293
eng,829241
fre,140055
```

### Änderung der Sortierreihenfolge

Standardmäßig wird die Häufigkeitsverteilung absteigend ausgegeben, d.h. der häufigste
Wert steht in der Ausgabe oben. Soll das Verhalten so geändert werden, dass die Ausgabe
aufsteigend sortiert wird, kann dies mit der Option `--reverse` bzw. `-r` erfolgen. Das
folgende Kommando sucht nach den drei Satzarten, die am wenigsten vorkommen:

```bash
$ pica frequency -s --limit 2 --reverse tests/data/dump.dat.gz
Ts1,1
Tp1,2
```

### Änderung der Unicode-Normalform

Die Unicode-Normalform in der Ausgabe lässt sich durch die Option `--translit` ändern. Liegen
die Daten in NFD-Normalform vor und sollen in die NFC-Normalform transliteriert werden, kann
dies mit dem folgenden Kommando erfolgen:

```bash
$ pica frequency --translit nfc "002@.0" dump.dat.gz
Ts1,1
Tp1,2
```

Es werden die Normalformen NFC (`nfc`), NFD (`nfd`), NFKC (`nfkc`) und NFKD (`nfkd`)
unterstützt. Wird die Option nicht verwendet, werden die Wertausprägungen in die Kodierung und
Normalform ausgegeben, wie sie in der Eingabedatei vorliegt.
