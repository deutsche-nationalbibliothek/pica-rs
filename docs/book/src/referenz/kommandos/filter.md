# filter

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Mithilfe des `filter`-Kommandos können Datensätze anhand eines Kriterums
aus der Eingabe gefiltert werden.


## Beschreibung

Das `filter`-Kommando bildet das Herzstück des Toolkits. Es ermöglicht
es, aus einer (mitunter sehr großen) Datenmenge, bspw. dem Gesamtabzug
des Katalogsystems, eine kleinere Menge effizient zu extrahieren, um sie
anschließend weiterzuverarbeiten. Dies erfolgt über die Angabe eines
Filterausdrucks, der darüber entscheidet, ob ein Datensatz in die
Zielmenge eingeht oder nicht.

Im folgenden Beispiel werden alle Datensätze aus der Datei
`DUMP.dat.gz` extrahiert, die ein Feld `003@` enthalten, das ein
Unterfeld `0` besitzt, welches mit dem Wert `118540238` belegt ist.

```console
$ pica filter -s '003@.0 == "118540238"' DUMP.dat.gz -o goethe.dat
$ pica print goethe.dat
001A $0 1250:01-07-88
001B $0 9999:15-04-22 $t 15:15:00.000
001D $0 0292:01-08-19
001U $0 utf8
001X $0 0
002@ $0 Tpz
003@ $0 118540238
003U $a http://d-nb.info/gnd/118540238 $z http://d-nb.info/gnd/185808069 $z http://d-nb.info/gnd/185848826 $z http://d-nb.info/gnd/101488358X $z http://d-nb.info/gnd/1014927390 $z http://d-nb.info/gnd/1022736213 $z http://d-nb.info/gnd/1095607278 $z http://d-nb.info/gnd/1131918517
004B $a piz
006Y $S isni $0 0000 0001 2099 9104
006Y $S wikidata $0 Q5879
...
028A $d Johann Wolfgang $c von $a Goethe
...
```

## Optionen

* `-s`, `--skip-invalid` — Überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-v`, `--invert-match` — Gibt alle Datensätze aus, die **nicht** dem
  Filterausdruck entsprechen.
* `-i`, `--ignore-case` — Groß- und Kleinschreibung wird bei Vergleichen
  ignoriert.
* `--strsim-threshold <value>` — Festlegen des Schwellenwerts beim
  Ähnlichkeitsvergleich von Zeichenketten mittels `=*`.
* `-k`, `--keep` — Es werden nur die Felder eines Datensatzes
  beibehalten, die in der Liste aufgeführt werden.
* `-d`, `--discard` — Es werden die Felder eines Datensatzes verworfen,
    die in der Liste aufgeführt werden.
* `-F`, `--file` `<file>` — Es wird der Filterausdruck aus der Datei
  `<file>` eingelesen. Es darf **keine** weitere Angabe eines
  Filterausdrucks als Kommandozeilenargument erfolgen!
* `-A`, `--allow-list` `<file>` — Es werden alle Datensätze ignoriert,
  die nicht explizit in der Positivliste[^1] auftauchen. Werden mehrere
  Positivlisten angegeben, wird die Mengenvereinigung aus allen Listen
  gebildet.
* `-D`, `--deny-list` `<file>` — Es werden alle Datensätze ignoriert,
  die in der Negativliste[^1] auftauchen. Werden mehrere Negativlisten
  angegeben, wird die Mengenvereinigung aus allen Listen gebildet.
* `-l`, `--limit` `<n>` — Eingrenzung der Ausgabe auf die ersten `<n>`
  (gültigen) Datensätze.
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
* `--append` — Wenn die Ausgabedatei bereits existiert, wird die
  Ausgabe an die Datei angehangen. Ist das Flag nicht gesetzt, wird eine
  bestehende Datei überschrieben.
* `--tee <filename>` — Abzweigen der Ausgabe in eine zusätzliche Datei.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben. Endet der Dateiname mit dem Suffix `.gz`, wird
  die Ausgabe automatisch im gzip-Format komprimiert.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Die Optionen zum Ignorieren invalider Datensätze und zum Komprimieren
der Ausgabe lassen sich in der `Pica.toml` konfigurieren:

```toml
[filter]
skip-invalid = true
gzip = false
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.


## Beispiele

### Invertierte Treffermenge (_invert match_)

Mitunter ist es einfacher, einen Ausdruck zu formulieren, der alle
Datensätze umfasst, die nicht in der Treffermenge gewünscht sind. Durch
die Option `-v`/`--invert-match` werden dann nur die Datensätze
ausgegeben, die nicht dem Filterkriterum entsprechen.

Beispielweise enthält der Abzug `DUMP.dat.gz` verschiedene
Normdatensätze. Werden nur die Datensätze benötigt, die **nicht** vom
Satztyp Werk sind, ist es einfacher, zuerst nach den Werken zu suchen
und dann durch das Flag `-v` alle Datensätze zu erhalten, die nicht dem
Filterkriterium entsprechen.

```console
$ pica filter -s -v '002@.0 =^ "Tu"' DUMP.dat.gz -o not-Tu.dat.gz
$ pica frequency '002@.0' not-Tu.dat.gz
Tsz,2
Tg1,1
Tp1,1
Tpz,1
Ts1,1

```

### Groß- und Kleinschreibung ingorieren

Standardmäßig wird bei Vergleichen von Zeichenketten die Groß- und
Kleinschreibung beachtet. Dies lässt sich mit dem Flag
`-i`/`--ignore-case` deaktivieren:

```console
$ pica filter -s -i '028A.a == "GOETHE"' DUMP.dat.gz -o goethe.dat
$ pica print goethe.dat
...
028A $d Johann Wolfgang $c von $a Goethe
...
```

### Felder eines Datensatzes reduzieren

Teilweise ist die Anzahl der Felder pro Datensatz sehr groß, was zu
erheblichen Laufzeiteinbußen nachfolgender Datenanalysen führen kann.
Mittels der Optionen `-k`/`--keep` bzw. `-d`/`--discard` lassen sich
Datensätze auf eine Teilmenge der Felder reduzieren.

Werden für eine Datenanalyse nur die IDN/PPN (`003@`), die Satzart
(`002@`) und die Entitätenkodierung (`004B`) benötigt, können die
Datensätze wie folgt auf die Felder reduziert werden:

```console
$ pica filter -s --keep '00[23]@,004B' "003@?" DUMP.dat.gz -o out.dat
$ pica print out.dat
002@ $0 Tpz
003@ $0 118540238
004B $a piz

002@ $0 Tp1
003@ $0 118607626
004B $a piz
...
```

Sollen bestimmte Felder entfernt werden, kann dies mit der Option
`-d`/`--discard` erfolgen. Der folgende Aufruf entfernt das Feld `004B`,
sofern vorhanden, aus allen Datensätzen:

```console
$ pica filter -s --keep '00[23]@,004B' "003@?" DUMP.dat.gz -o out.dat
$ pica filter --discard '004B' "003@?" out.dat -o out2.dat
$ pica print out2.dat
002@ $0 Tpz
003@ $0 118540238

002@ $0 Tp1
003@ $0 118607626
...
```



[^1]: Eine Positiv- oder Negativliste muss entweder als [CSV]-Datei
    vorliegen, die in der **ersten Spalte** die IDN/PPN des Datensatzes
    enthält, oder als eine [Arrow]-Datei, die eine `idn`-Spalte enthält.
    Alle Dateien werden automatisch als [CSV]-Datei interpretiert, es
    sei denn, die Datei endet mit `.ipc`, `.arrow` oder `.feather`, dann
    erfolgt die Interpretation im [Arrow]-Format.

[Arrow]: https://arrow.apache.org/
[CSV]: https://de.wikipedia.org/wiki/CSV_(Dateiformat)
[Gzip]: https://de.wikipedia.org/wiki/Gzip
