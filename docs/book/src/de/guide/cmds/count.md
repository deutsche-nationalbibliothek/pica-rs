# count

> Hinweis: Das Kommando befindet sich im Beta-Status und wird gerade intensiv getestet, bevor
> es als stabil freigegeben wird. Änderungen am _command-line interface_ (CLI) sowie das
> Auftreten kleinerer Fehler ist möglich.

Mithilfe des `count`-Kommandos lässt sich die Anzahl an Datensätzen (_records_), Feldern (_fields_)
sowie Unterfeldern (_subfields_) ermitteln.

## Beschreibung

Soll die Anzahl der Datensätze und deren Felder sowie Unterfelder ermittelt werden, kann dies mit
dem `count`-Kommando erfolgen. Ungültige Datensätze können mit dem Flag `--skip-invalid` (bzw.
`-s`) übersprungen werden. Im folgenden Beispiel werden drei Dateien eingelesen und eine
Zusammenfassung auf der Konsole ausgegeben:

```bash
$ pica count -s 004732650.dat.gz 1004916019.dat.gz 119232022.dat.gz
records: 3
fields: 122
subfields: 332
```

### Ausgabe im CSV/TSV-Format

Die Ausgabe des Kommandos kann auch im Format _CSV_ bzw. _TSV_ erfolgen, was die Weiterverarbeitung
in anderen Programmen erleichtert. Die Ausgabe der Kopfzeile lässt sich mit dem Flag `--no-header`
ausschalten.

```bash
$ pica count -s --csv tests/data/dump.dat.gz
records,fields,subfields
7,247,549

$ pica count -s --tsv tests/data/dump.dat.gz
records fields  subfields
7       247     549

$ pica count -s --csv --no-header tests/data/dump.dat.gz
records,fields,subfields
7,247,549
```

### Ausgabe in eine Datei

Die Ausgabe des Kommandos wird standardmäßig auf der Konsole ausgegeben. Diese kann mit der Option
`--output` (bzw. `-o`) in eine Datei umgeleitet werden. Soll diese Datei nicht bei jedem Aufruf
überschrieben werden, sondern bei jedem Aufruf eine neue Zeile erhalten, kann dies mit dem Flag
`--append` erzielt werden.

```bash
$ pica count -s --csv -o count.csv tests/data/dump.dat.gz
$ cat count.csv
records,fields,subfields
7,247,549

$ pica count -s --csv --append -o count.csv tests/data/dump.dat.gz
$ cat count.csv
records,fields,subfields
7,247,549
7,247,549
```

### Ausgabe von Einzelwerten

Soll entweder die Anzahl von Datensätzen, Feldern oder Unterfeldern ausgegeben werden, kann dies
mit den Flags `--records`, `--fields` bzw. `--subfields` erfolgen. Diese Flags schließen sich
gegenseitig aus und können nicht mit den Flags `--csv`, `--tsv` und `--no-header` kombiniert
werden.

```bash
$ pica count -s --records tests/data/dump.dat.gz
7

$ pica count -s --fields tests/data/dump.dat.gz
247

$ pica count -s --subfields tests/data/dump.dat.gz
549
```

### Anwendungsbeispiel

Soll die Veränderung (Anzahl Datensätze, Felder, Unterfelder) eines Abzugs über die Zeit verfolgt
werden, könnte dies wie folgt erreicht werden:

```bash
$ echo "date,records,fields,subfields" > count.csv # Kopfzeile
$ pica count -s dump.dat.gz --append -o count.csv # Initialer Aufruf
$ pica count -s dump.dat.gz --append -o count.csv # Aufruf nach x Tagen

$ cat count.csv
$ records,fields,subfields
7,247,549
9,347,1022
```

Soll auch das aktuelle Datum vor die Zeile geschrieben werden, könnten die Befehle wie folgt
aussehen:

```bash
# Schreiben der Kopfzeile
$ echo "date,records,fields,subfields" > count.csv

# Aufruf am 22.02.2022
$ pica count -s --no-header --csv dump.dat.gz | \
    xargs -d"\n" -I {} date +"%Y-%m-%d,{}" >> count.csv

# Aufruf am 23.02.2022
$ pica count -s --no-header --csv dump.dat.gz | \
    xargs -d"\n" -I {} date +"%Y-%m-%d,{}" >> count.csv

$ cat count.csv
$ date,records,fields,subfields
2022-02-22,7,247,549
2022-02-23,9,347,1022
```

### Alternativen

Da Datensätze zeilenweise gespeichert/ausgegeben werden, kann auch das Unix-Kommand [_wc_] verwendet werden,
um die Anzahl der Datensätze zu bestimmen. Es muss aber vorher sichergestellt sein, dass nur gültige Datensätze
in der Datei (oder der Standardeingabe) sind. Die folgenden Kommandos sind äquivalent:

```bash
$ pica count -s --records dump.dat
7

$ pica cat -s dump.dat | wc -l
7
```

Eine weitere Alternative ist das Perl-Tool [_picadata_]. Es kann wie folgt genutzt werden (eine genaue
Beschreibung des Tools befindet sich in der Dokumentation):

```bash
$ pica filter "045H?" ddc.pica | pica count
719229 records
1723512 fields
```

[_wc_]: https://man7.org/linux/man-pages/man1/wc.1.html
[_picadata_]: https://metacpan.org/dist/PICA-Data/view/script/picadata
