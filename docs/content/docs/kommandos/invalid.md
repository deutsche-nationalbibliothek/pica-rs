# invalid

Das `invalid`-Kommando findet Zeilen in der Eingabe, die nicht als
normalisiertes PICA+ dekodiert werden konnten und somit ungültig sind.

## Beschreibung

Bei der Verarbeitung von PICA-Daten kann es vorkommen, dass Zeilen in
der Eingabe nicht dekodiert werden können. Diese ungültigen Zeilen
lassen sich bei vielen Kommandos mit der Option `--skip-invalid` (bzw.
`-s`) überspringen, wobei die Anzahl der übersprungenen Zeilen nicht
angezeigt wird. Es kann aber empfehlenswert oder notwendig sein, die
Anzahl invalider Datensätze zu kontrollieren und einer Prüfung zu
unterziehen, um diese ggf. zu korrigieren.

Der folgende Befehl findet alle ungültigen Datensätze aus der Datei
`DUMP.dat.gz` und schreibt diese Zeile in die Datei `invalid.dat`:

```console
$ pica invalid DUMP.dat.gz -o invalid.dat
```

## Optionen

* `-p`, `--progress` — Anzeige des Fortschritts, der die Anzahl der
  eingelesenen gültigen sowie invaliden Datensätze anzeigt. Das
  Aktivieren der Option erfordert das Schreiben der Datensätze in eine
  Datei mittels `-o` bzw. `--output`.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.


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
