# count

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Mithilfe des `count`-Kommandos lässt sich die Anzahl an Datensätzen
(_records_), Feldern (_fields_) sowie Unterfeldern (_subfields_)
ermitteln.

## Beschreibung

Soll die Anzahl der Datensätze und deren Felder sowie Unterfelder
ermittelt werden, kann dies mit dem `count`-Kommando erfolgen. Ungültige
Datensätze können mit dem Flag `--skip-invalid` (bzw. `-s`) übersprungen
werden. Im folgenden Beispiel werden drei Dateien eingelesen und eine
Zusammenfassung auf der Konsole ausgegeben:

```bash
$ pica count -s 004732650.dat.gz 1004916019.dat.gz 119232022.dat.gz
records: 3
fields: 122
subfields: 332
```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `--append` — Wenn die Ausgabedatei bereits existiert, wird die
  Ausgabe an die Datei angehangen. Ist das Flag nicht gesetzt, wird eine
  bestehende Datei standardmäßig überschrieben.
* `--records` — gibt nur die Anzahl der vorhandenen Datensätze aus.
  Dieses Flag ist nicht mit den Optionen `--fields`, `--subfields`,
  `--csv`, `--tsv` und `--no-header` kombinierbar.
* `--fields` — gibt nur die Anzahl der vorhandenen Felder aus.
  Dieses Flag ist nicht mit den Optionen `--records`, `--subfields`,
  `--csv`, `--tsv` und `--no-header` kombinierbar.
* `--subfields` — gibt nur die Anzahl der vorhandenen Unterfelder aus.
  Dieses Flag ist nicht mit den Optionen `--records`, `--fields`,
  `--csv`, `--tsv` und `--no-header` kombinierbar.
* `--csv` — die Ausgabe erfolgt im CSV-Format.
* `--tsv` — die Ausgabe erfolgt im TSV-Format.
* `--no-header` — es wird keine Kopfzeile bei der CSV- bzw- TSV-Ausgabe
  geschrieben.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.

## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Die Option zum Ignorieren invalider Datensätze lässt sich in der
`Pica.toml` konfigurieren:

```toml
[count]
skip-invalid = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.

## Beispiele

### Ausgabe im CSV/TSV-Format

Die Ausgabe des Kommandos kann auch im Format _CSV_ bzw. _TSV_ erfolgen,
was die Weiterverarbeitung in anderen Programmen erleichtert. Die
Ausgabe der Kopfzeile lässt sich mit dem Flag `--no-header` ausschalten.

```bash
$ pica count -s --csv tests/data/dump.dat.gz
records,fields,subfields
7,247,549

$ pica count -s --tsv tests/data/dump.dat.gz
records fields  subfields
7       247     549

$ pica count -s --csv --no-header tests/data/dump.dat.gz
7,247,549
```

### Ausgabe in eine Datei

Die Ausgabe des Kommandos wird standardmäßig auf der Konsole ausgegeben.
Diese kann mit der Option `--output` (bzw. `-o`) in eine Datei
umgeleitet werden. Soll diese Datei eine neue Zeile erhalten und nicht
bei jedem Aufruf überschrieben werden, kann dies mit dem Flag `--append`
erzielt werden.

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

Soll entweder die Anzahl von Datensätzen, Feldern oder Unterfeldern
ausgegeben werden, kann dies mit den Flags `--records`, `--fields` bzw.
`--subfields` erfolgen. Diese Flags schließen sich gegenseitig aus und
können nicht mit den Flags `--csv`, `--tsv` und `--no-header` kombiniert
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

Soll die Veränderung (Anzahl Datensätze, Felder, Unterfelder) eines
PICA-Abzugs über die Zeit verfolgt werden, könnte dies wie folgt
erreicht werden:

```bash
$ echo "date,records,fields,subfields" > count.csv # Kopfzeile
$ pica count -s dump_20220222.dat.gz --append -o count.csv # Initialer Aufruf
$ pica count -s dump_20220223.dat.gz --append -o count.csv # Aufruf nach x Tagen

$ cat count.csv
$ records,fields,subfields
7,247,549
9,347,1022
```

Soll auch das aktuelle Datum vor die Zeile geschrieben werden, könnten
die Befehle wie folgt aussehen:

```bash
# Schreiben der Kopfzeile
$ echo "date,records,fields,subfields" > count.csv

# Aufruf am 22.02.2022
$ pica count -s --no-header --csv dump_20220222.dat.gz | \
    xargs -d"\n" -I {} date +"%Y-%m-%d,{}" >> count.csv

# Aufruf am 23.02.2022
$ pica count -s --no-header --csv dump_20220223.dat.gz | \
    xargs -d"\n" -I {} date +"%Y-%m-%d,{}" >> count.csv

$ cat count.csv
$ date,records,fields,subfields
2022-02-22,7,247,549
2022-02-23,9,347,1022
```

## Alternative

Da Datensätze zeilenweise gespeichert/ausgegeben werden, kann auch das
Unix-Kommand [_wc_] verwendet werden, um die Anzahl der Datensätze zu
bestimmen. Es muss aber vorher sichergestellt sein, dass nur gültige
Datensätze in der Datei (oder der Standardeingabe) sind. Die folgenden
Kommandos sind äquivalent:

```bash
$ pica count -s --records dump.dat
7

$ pica cat -s dump.dat | wc -l
7
```

[_wc_]: https://man7.org/linux/man-pages/man1/wc.1.html
