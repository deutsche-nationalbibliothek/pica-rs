# frequency

Mithilfe des Kommandos `frequency` lässt sich die Häufigkeitsverteilung
aller Wertausprägungen eines Unterfelds bestimmen.

## Beschreibung

Das Kommando `frequency` wird dazu genutzt, um die Häufigkeiten der
Wertausprägungen eines Unterfelds zu bestimmen. Ist das zu untersuchende
Feld bzw. Unterfeld wiederholbar, dann gehen alle Wertausprägungen eines
Datensatzes in die Häufigkeitsverteilung ein. Aus der Ergebnisdatei lässt
sich dann durch geeignete Tools eine grafische Darstellung (Histogramm)
oder die relative Häufigkeitsverteilung berechnen. Die Ausgabe erfolgt
standardmäßig im CSV-Format.

Im folgenden Beispiel wird die Häufigkeitsverteilung des Unterfelds
`010@.a` (Sprache des Textes) ermittelt. Sowohl das Feld `010@` als auch
das Unterfeld `a` sind wiederholbar und somit werden Datensätze, die sowohl
einen Sprachencode `ger` als auch `eng` erhalten haben, für jeden Wert
gezählt.

```bash
$ pica frequency "010@.a" FILE.dat
ger,2888445
eng,347171
...
```

### Hinzufügen einer Kopfzeile

Für die Dokumentation sowie die Verwendung in anderen Programmiersprachen
ist es häufig sinnvoll eine Kopfzeile hinzuzufügen. Dies erfolgt mit der
Option `--header` bzw. `-H`. Die Namen der Spalten werden komma-separiert
angegeben. Die Angabe von mehr als zwei Spalten ist nicht erlaubt.

```bash
$ pica frequency --header "sprache,anzahl" "010@.a" A.dat
sprache,anzahl
ger,2888445
eng,347171
...
```

### Eingrenzung auf bestimmte Felder

Oftmals sollen nicht alle Felder in die Berechnung der Häufigkeiten mit
einbezogen werden. Dies ist bspw. dann der Fall, wenn sich Felder anhand
eines Unterfelds unterschieden lassen, wie etwa durch die Angabe der
Metadatenherkunft. Durch Verwenden eines Pfad-Ausdrucks in {}-Notation,
können nur die Felder ausgewählt werden, die einem bestimmten Kriterium
entsprechen.

Im folgenden Beispiel werden von einem Datensatz nur die `044H` Felder in
die Ergenisbereichnung mit einbezogen, die ein Unterfeld `b` besitzen, das
gleich `'GND'` ist, sowie ein Unterfeld `H`, das mit der Zeichenkette
`'ema'` beginnt. Felder, die nicht dem Filter entsprechen werden ignoriert.

```bash
$ pica frequency "044H{ 9 | b == 'GND' && H =^ 'ema' }" DUMP.dat
gnd_id,count
040118827,29359
040305503,4118
041132920,2861
04061963X,2420
040288595,1964
...
```

Mit der Option `--ignore-case` (bzw. `-i`) wird bei Vergleichen von Werten
die Groß-/Klein-Schreibung ignoriert. Die Option `--strsim-threshold` legt
den Schwellenwert des `=*`-Operators fest, mit dem auf die Ähnlichkeit von
Zeichenketten geprüft werden kann.


### Eingrenzen der Treffermenge (Limit)

Soll die Treffermenge auf die _n_-häufigsten Werte eingeschränkt werden,
wird dies mit der Option `--limit` bzw. `-l` erreicht. Das folgende
Beispiel sucht nach den fünf häufigsten Sprachencodes:

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

Die Treffermenge kann auch mittels der Angabe eines Schwellenwerts
eingeschänkt werden. Sollen nur die Werte angezeigt werden, die häufiger
als ein Schwellenwert _n_ vorkommen, dann kann dies mit der Option
`--threshold` bzw. `-t` erzielt werden:

```bash
$ pica frequency --theshold 100000 -H "sprache,anzahl" "010@.a" A.dat
sprache,anzahl
ger,4944293
eng,829241
fre,140055
```

### Änderung der Sortierreihenfolge

Standardmäßig wird die Häufigkeitsverteilung absteigend ausgegeben,
d.h. der häufigste Wert steht in der Ausgabe oben[^fn1]. Soll das
Verhalten so geändert werden, dass die Ausgabe aufsteigend sortiert wird,
kann dies mit der Option `--reverse` bzw. `-r` erfolgen. Das folgende
Kommando sucht nach den drei Satzarten, die am wenigsten vorkommen:

```bash
$ pica frequency -s --limit 2 --reverse tests/data/dump.dat.gz
Ts1,1
Tp1,2
```

### Ausgabe im TSV-Format

Die Ausgabe lässt sich mittels der Option `--tsv` (bzw. `-t`) in das TSV-
Format ändern.

```bash
$ pica frequency -s --tsv --reverse tests/data/dump.dat.gz
Tp1    2
Ts1    1
...
```

### Änderung der Unicode-Normalform

Die Unicode-Normalform in der Ausgabe lässt sich durch die Option
`--translit` ändern. Liegen die Daten in NFD-Normalform vor und sollen in
die NFC-Normalform transliteriert werden, kann dies mit dem folgenden
Kommando erfolgen:

```bash
$ pica frequency --translit nfc "002@.0" dump.dat.gz
Ts1,1
Tp1,2
```

Es werden die Normalformen NFC (`nfc`), NFD (`nfd`), NFKC (`nfkc`) und
NFKD (`nfkd`) unterstützt. Wird die Option nicht verwendet, werden die
Wertausprägungen in die Kodierung und Normalform ausgegeben, wie sie in
der Eingabedatei vorliegt.


[^fn1]: Alle Werte mit gleicher Häufigkeit werden immer in lexikographisch
    aufsteigender Reihenfolge sortiert. Dies erfolgt unabhängig vom
    Parameter `--reverse`.
