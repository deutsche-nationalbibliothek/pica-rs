# hash

Mithilfe des Kommandos `hash` lässt sich eine Tabelle erzeugen, die in
der ersten Spalte die IDN (Feld `003@.0`) eines Datensatzes und in der
zweiten Spalte den [_SHA-256_] Hashwert enthält.

## Beschreibung

Mitunter kommt es vor, dass eine regelmäßige und aufwändige Berechnung
für Datensätze ausgeführt werden muss und es nicht praktikabel ist, die
Berechnung über alle Datensätze durchzuführen. Mittels des
`hash`-Kommandos können auf unterschiedlichen Abzügen die Hashwerte der
Datensätze bestimmt werden. Durch Vergleich dieser Tabelle ist es
möglich, die Datensätze zu identifizieren, die sich geändert haben,
gelöscht oder neu hinzugefügt wurden. Das folgende Beispiel demonstriert
die Erzeugung dieser Hash-Tabellen:

```console
$ pica hash -s DUMP.dat.gz
idn,sha256
118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3
118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee
040993396,0361c33e1f7a80e21eecde747b2721b7884e003ac4deb8c271684ec0dc4d059a
...
```


## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-H`, `--header` `<header>` — Kopfzeile, die den Ergebnissen
  vorangestellt wird.
* `-t`, `--tsv` — Ausgabe erfolgt im TSV-Format.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Die Option zum Ignorieren invalider Datensätze lässt sich in der
`Pica.toml` konfigurieren:

```toml
[hash]
skip-invalid = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.


## Beispiele

### Ausgabe im TSV-Format

Die Ausgabe lässt sich mittels der Option `--tsv` (bzw. `-t`) in das
TSV-Format ändern.

```console
$ pica hash -s --tsv DUMP.dat.gz
idn	sha256
118540238	762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3
118607626	8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee
040993396	0361c33e1f7a80e21eecde747b2721b7884e003ac4deb8c271684ec0dc4d059a
...
```

### Hinzufügen einer Kopfzeile

Eine individuelle Kopfzeile lässt sich mit der Option `--header` (bzw.
`-H`) ausgeben:

```console
$ pica hash -s --header 'ppn,hash' DUMP.dat.gz
ppn,hash
118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3
118607626,8d75e2cdfec20aa025d36018a40b150363d79571788cd92e7ff568595ba4f9ee
040993396,0361c33e1f7a80e21eecde747b2721b7884e003ac4deb8c271684ec0dc4d059a
...
```


## Anmerkung

Zur Berechnung des [_SHA-256_]-Hashwerts wird der abschließende
Zeilenumbruch mit einbezogen, um einen gleichen Hashwert zu erzeugen,
der entsteht, wenn der Hashwert über die gesamte Zeile aus der Eingabe
ermittelt wird. Im folgenden Beispiel wird zuerst der erste gültige
Datensatz in die Datei `1.dat` geschrieben. Anschließend wird der
Hashwert einmal mit dem `hash`-Kommando und einmal mit dem Unix-Programm
[_sha256sum_] gebildet. Beide Ergebnisse sind gleich.

```console
$ pica filter -s -l1 "003@?" DUMP.dat.gz -o 1.dat
$ pica hash 1.dat
idn,sha256
118540238,762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3

$ sha256sum 1.dat
762cf3a1b18a0cad2d0401cd2b573a89ff9c81b43c4ddab76e136d7a10a851f3
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

[_SHA-256_]: https://de.wikipedia.org/wiki/SHA-2
[_sha256sum_]: https://manpages.ubuntu.com/manpages/trusty/de/man1/sha256sum.1.html
