# `explode`

![stability-badge](https://img.shields.io/badge/stability-unstable-red?style=flat-square)

Mithilfe des `explode`-Kommandos lassen sich Datensätze in Lokal- bzw.
Exemplardatensätze aufteilen.

> **Hinweis:** Das `explode`-Kommando befindet sich in der aktiven
> Entwicklung. Funktionalitäten können unvollständig oder fehlerhaft
> sein. Änderungen am _command-line interface_ (CLI) sind nicht
> ausgeschlossen.

## Beschreibung

Sollen Datenanalysen auf der Ebene von Lokal- bzw. Exemplardatensätzen
durchgeführt werden, müssen diese zunächst mit dem `explode`-Kommando
aufgeteilt werden. Das Aufteilen der Datensätze erfolgt durch die Angabe
des _Levels_. Mögliche Werte sind: _main_, _local_ und _copy_. Die
Auswahl des _main_-Levels hat keinen Effekt, der Datensatz wird nicht
aufgeteilt und wie er ist in die Ausgabe geschrieben. 

Mit der Auswahl des _local_-Levels wird ein Datensatz anhand der
Lokaldatensatz-ID (_ILN_, `101@.a`) aufgeteilt. Jeder Datensatz enthält
die Felder der Ebene 0, die jeweilige Lokaldatensatz-ID und alle
dazugehörigen Examplare.

Schließlich kann durch Auswahl des _copy_-Levels für jedes Examplar ein
eigenständiger PICA-Datensatz erzeugt werden, der alle Felder der Ebene
0 sowie die dazugehörige Lokaldatensatz-ID enthält.

```console,ignore
$ pica explode -s copy --where '101@.a == "1"' DUMP.dat.gz
```

