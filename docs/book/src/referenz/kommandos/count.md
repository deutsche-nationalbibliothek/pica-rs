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

```console
$ pica count -s DUMP.dat.gz
records: 12
fields: 1035
subfields: 3973

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
* `--no-header` — es wird keine Kopfzeile in die Ausgabe geschrieben.
* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
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

```console
$ pica count -s --csv DUMP.dat.gz
records,fields,subfields
12,1035,3973

$ pica count -s --tsv DUMP.dat.gz
records	fields	subfields
12	1035	3973

$ pica count -s --csv --no-header DUMP.dat.gz
12,1035,3973

```

### Ausgabe in eine Datei

Die Ausgabe des Kommandos wird standardmäßig auf der Konsole ausgegeben.
Diese kann mit der Option `--output` (bzw. `-o`) in eine Datei
umgeleitet werden. Soll diese Datei eine neue Zeile erhalten und nicht
bei jedem Aufruf überschrieben werden, kann dies mit dem Flag `--append`
erzielt werden.

```console
$ pica count -s --csv -o count.csv DUMP.dat.gz
$ cat count.csv
records,fields,subfields
12,1035,3973

$ pica count -s --csv --append -o count.csv DUMP.dat.gz
$ cat count.csv
records,fields,subfields
12,1035,3973
12,1035,3973
```

### Ausgabe von Einzelwerten

Soll entweder die Anzahl von Datensätzen, Feldern oder Unterfeldern
ausgegeben werden, kann dies mit den Flags `--records`, `--fields` bzw.
`--subfields` erfolgen. Diese Flags schließen sich gegenseitig aus und
können nicht mit den Flags `--csv`, `--tsv` und `--no-header` kombiniert
werden.

```console
$ pica count -s --records DUMP.dat.gz
12

$ pica count -s --fields DUMP.dat.gz
1035

$ pica count -s --subfields DUMP.dat.gz
3973

```

### Anwendungsbeispiel

Soll die Veränderung (Anzahl Datensätze, Felder, Unterfelder) eines
PICA-Abzugs über die Zeit verfolgt werden, könnte dies wie folgt
erreicht werden:

```console,ignore
$ echo "date,records,fields,subfields" > count.csv # Kopfzeile
$ pica count -s dump_20220222.dat.gz --append -o count.csv # Initialer Aufruf
$ pica count -s dump_20220223.dat.gz --append -o count.csv # Aufruf nach x Tagen

$ cat count.csv
$ records,fields,subfields
7,247,549
9,347,1022
```

Soll auch das aktuelle Datum vor die Zeile geschrieben werden, könnten
bspw. folgende Unix-Kommandos genutzt werden:

```console,ignore
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
