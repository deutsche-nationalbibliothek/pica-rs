# invalid

Das Kommando `invalid` filtert aus einer Datei Zeilen heraus, die nicht
dekodiert werden konnten und somit ungültig sind.

## Beschreibung

Bei der Verarbeitung von PICA-Daten kann es vorkommen, dass Zeilen in
der Eingabe(-datei) nicht dekodiert werden können. Diese ungültigen Zeilen
lassen sich bei den vielen Kommandos mit der Option `--skip-invalid` (bzw.
`-s`) überspringen, wobei die Anzahl der übersprungenen Zeilen nicht
angezeigt wird. Es kann aber empfehlenswert oder notwendig sein, die Anzahl
invalider Datensätze zu kontrollieren und einer Prüfung zu unterziehen, um
diese ggf. zu korrigieren.

Im folgenden Befehl werden alle ungültigen Datensätze aus der Datei
`DUMP.dat.gz` gefiltert und in die Datei `invalid.dat` geschrieben.

```bash
$ pica invalid DUMP.dat.gz -o invalid.dat
```




