# explode

Mithilfe des `explode`-Kommandos lassen sich Datensätze in Lokal- bzw.
Exemplardatensätze aufteilen.

{{< hint danger >}}
**Hinweis:**
Das `explode`-Kommando befindet sich in der aktiven Entwicklung.
Funktionalitäten können unvollständig oder fehlerhaft sein. Änderungen
am _command-line interface_ (CLI) sind nicht ausgeschlossen.
{{< /hint >}}

## Beschreibung

<!-- TODO: Separate Seite über den Aufbau eines PICA+-Datensatzes erstellen. Untenstehende Notizen dazu stammen aus https://wiki.k10plus.de/download/attachments/203128864/K10plusKatalogisierungsschulung-mit-Notizen-Teil1.pdf?version=3&modificationDate=1606824839604&api=v2 -->

Die Verarbeitung und Analyse von Datensätzen auf Lokal- bzw.
Exemplarebene ist mitunter nur unzureichend möglich, da Filterausdrücke
die Grenzen von untergeordneten Ebenen nicht respektiert. Abhilfe kann
das `explode`-Kommando schaffen, das einen Datensatz in einzelne Lokal-
bzw. Exemplardatensätze aufteilen kann. Dabei werden alle Felder der
übergeordneten Ebenen mit in die Datensätze übernommen.

Das Aufteilen der Datensätze erfolgt durch die Angabe der Ebene
(_level_) an der der Datensatz geteilt werden soll. Es können folgende
Werte ausgewählt werden:

* `main` (Aufteilen auf Ebene der Titeldaten),
* `local` (Aufteilen auf Ebene der Lokaldaten),
* sowie `copy` (Aufteilen auf Ebene der Exemplardaten).

Soll ein Datensatz in alle Lokaldatensätze aufgeteilt werden, muss die
Ebene `local` ausgewählt werden. Die neu erstellten Datensätze enthalten
alle Titeldaten (Felder der Ebene main), den Identifikator des
Lokaldatensatzes (Feld `101@.a`) sowie alle Exemplare, die diesem
Lokaldatensatz zugeordnet werden.

Soll darüber hinaus für jedes Exemplar ein eigenständiger Datensatz
erzeugt werden, muss die Ebene `copy` angegeben werden. Jeder erzeugte
Datensatz enthält die Titeldaten (Felder der Ebene main), den Identifikator
des Lokaldatensatzes (Feld `101@.a`) und nur die Felder, die zum
jeweiligen Exemplar gehören.

Schließlich kann ein Datensatz auch auf Ebene der Titeldaten (`main`)
aufgeteilt werden. Diese Aufwahl verändert nichts am Datensatz und gibt
den vollständigen Datensatz mit allen Feldern aus.

Als Beispiel soll folgender (reduzierter) Datensatz dienen:

```console
$ pica print COPY.dat.gz
003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/01 $0 0123456789
203@/02 $0 1234567890
101@ $a 2
203@/01 $0 345678901


```

Dieser Datensatz lässt sich in zwei Datensätze auf Ebene der Lokaldaten
aufteilen:

```console
$ pica explode -s local COPY.dat.gz -o local.dat
$ pica print local.dat
003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/01 $0 0123456789
203@/02 $0 1234567890

003@ $0 123456789
002@ $0 Abvz
101@ $a 2
203@/01 $0 345678901


```

Soll jedes Exemplar ein eigenständiger Datensatz werden, wird dies durch
Angabe von `copy` erzielt:

```console
$ pica explode -s copy COPY.dat.gz -o copy.dat
$ pica print copy.dat
003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/01 $0 0123456789

003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/02 $0 1234567890

003@ $0 123456789
002@ $0 Abvz
101@ $a 2
203@/01 $0 345678901


```


## Optionen

* `-s`, `--skip-invalid` — Überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-i`, `--ignore-case` — Groß- und Kleinschreibung wird bei Vergleichen
  ignoriert.
* `--strsim-threshold <value>` — Festlegen des Schwellenwerts beim
  Ähnlichkeitsvergleich von Zeichenketten mittels `=*`.
* `-l`, `--limit` `<n>` — Eingrenzung der Ausgabe auf die ersten `<n>`
  (gültigen) Datensätze.
* `--where` `<filter>` — Angabe eines Filters, der auf die erzeugten
  Datensätze angewandt wird.
* `--and` `<expr>` — Hinzufügen eines zusätzlichen Filters mittels der
  booleschen `&&`-Verknüpfung. Der ursprüngliche Filterausdruck
  `<filter>` wird zum Ausdruck `<filter> && <expr>`.
* `--or` `<expr>` — Hinzufügen eines zusätzlichen Filters mittels der
  booleschen `||`-Verknüpfung. Der ursprüngliche Filterausdruck
  `<filter>` wird zum Ausdruck `<filter> || <expr>`.
* `--not` `<expr>` — Hinzufügen eines zusätzlichen Filters. Der
  ursprüngliche Filterausdruck `<filter>` wird zum Ausdruck `<filter> &&
  !(<expr>)`.
* `-g`, `--gzip` — Komprimieren der Ausgabe im [Gzip]-Format.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben. Endet der Dateiname mit dem Suffix `.gz`, wird
  die Ausgabe automatisch im [Gzip]-Format komprimiert.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Einige Kommandozeilen-Optionen lassen sich per Konfigurationsdatei
(`Pica.toml`) einstellen:

```toml
[explode]
skip-invalid = true
gzip = true
```

## Beispiele

### Eingrenzen der Datensätze

Ist nur eine Teilmenge der erzeugten Datensätze von Interesse, lässt
sich die Ergebnismenge durch Hinzufügen eines Filterausdrucks
eingrenzen.

Werden bspw. nur die Exemplare mit dem Identifikator `101@.a == "1"`
benötigt, kann die Eingrenzung durch Angabe der `--where`-Option
eingegrenzt werden:

```console
$ pica explode -s copy --where '101@.a == "1"' COPY.dat.gz -o copy.dat
$ pica print copy.dat
003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/01 $0 0123456789

003@ $0 123456789
002@ $0 Abvz
101@ $a 1
203@/02 $0 1234567890


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

[Gzip]: https://de.wikipedia.org/wiki/Gzip
