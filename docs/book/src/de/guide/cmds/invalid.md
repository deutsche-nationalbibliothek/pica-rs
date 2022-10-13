# invalid

Das Kommando `invalid` filtert aus einer Datei Zeilen heraus, die nicht
dekodiert werden konnten und somit ungültig sind.

> Hinweis: Das Kommando befindet sich im Beta-Status und wird gerade
> intensiv getestet, bevor es als stabil freigegeben wird. Änderungen
> am _command-line interface_ (CLI) sowie das Auftreten kleinerer Fehler
> ist möglich.

## Beschreibung

Bei der Verarbeitung von PICA-Daten kann es vorkommen, dass Zeilen in
der Eingabe(-datei) nicht dekodiert werden können. Diese ungültigen Zeilen
lassen sich bei den vielen Kommandos mit der Option `--skip-invalid`
(bzw. `-s`) überspringen, wobei die Anzahl der übersprungenen Zeilen nicht
angezeigt wird. Deshalb empfiehlt es sich die Anzahl invalider Datensätze
im Auge zu behalten und einer Prüfung zu unterziehen, um diese ggf. zu
korrigieren.

Im folgenden Befehl werden alle ungültigen Datensätze aus der Datei
`DUMP.dat.gz` gefiltert und in die Datei `invalid.dat` geschrieben.

```bash
$ pica invalid DUMP.dat.gz -o invalid.dat
```




