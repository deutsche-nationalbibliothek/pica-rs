# cat

Das `cat`-Kommando liest Datensätze direkt von der Standardeingabe
(`stdin`) oder aus Dateien ein und fügt diese zusammen. Die Ausgabe
kann entweder in eine Datei oder in die Standardausgabe (`stdout`)
geschrieben werden.

## Beschreibung

Der wichtigste Anwendungsfall des Kommandos besteht in Kombination mit
den Kommandos _[partition]_ oder _[split]_, da mittels `cat` das
Ergebnis von _[partition]_ oder _[split]_ (teil-)rückgängig gemacht
werden kann. Häufig macht es Sinn, eine große Datei in viele kleinere
Dateien anhand eines Kriteriums zu teilen und anschließend eine
Teilmenge wieder zusammenzufügen.

Das folgende Beispiel fügt die zwei Partitionen `ger.dat` und `eng.dat`
zu einer Datei zusammen:

```bash
$ pica cat ger.dat eng.dat -o ger_eng.dat
```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-u`, `--unique` — es werden keine Duplikate in die Ausgabe
  geschrieben. Die Strategie zur Erkennung von Duplikaten wird mittels
  der Option `--unique-strategy` festgelegt.
* `--unique-strategy <strategy>` — festlegen, wie Duplikate erkannt
  werden sollen. Standardmäßig ist der Wert `idn` gesetzt und es werden
  Duplikate durch Vergleichen der PPN/IDN (Feld `003@.0`) eines
  Datensatzes erkannt. Alternativ kann die Strategie `hash` gewählt
  werden. Der Vergleich erfolgt dann über die SHA-256 Prüfsumme der
  Datensätze.
* `--append` — Wenn die Ausgabedatei bereits existiert, wird die
  Ausgabe an die Datei angehangen. Ist das Flag nicht gesetzt, wird eine
  bestehende Datei standardmäßig überschrieben.
* `--tee <filename>` — Abzweigen der Ausgabe in eine zusätzliche Datei.
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

Die Optionen zum Ignorieren invalider Datensätze sowie das Komprimieren
der Ausgabe lassen sich in der `Pica.toml` konfigurieren:

```toml
[cat]
skip-invalid = true
unique-strategy = "hash"
gzip = false
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.

## Beispiele


### Überspringen ungültiger Datensätze

Der eingangs verwendete Befehl geht davon aus, dass die zwei Partitionen
ausschließlich gültige Datensätze enthalten. Gültig in diesem
Zusammenhang bedeutet, dass es sich um valide Datensätze im Format PICA+
handelt und nicht ob ein Datensatz einem bestimmten Regelwerk
entspricht.

Das Ausschließen von ungültigen Datensätzen wird mit der Option
`--skip-invalid` oder `-s` erreicht:

```bash
$ pica cat --skip-invalid DUMP.dat.gz -o dump_valid.dat
$ pica cat -s DUMP.dat.gz --output dump_valid.dat.gz
```

### Lesen von der Standardeingabe

Das Kommando kann auch direkt von der Standardeingabe (`stdin`) lesen.
Das ist bspw. dann hilfreich, wenn die Ausgabe aus einem vorhergehenden
Pipeline-Schritt mit dem Inhalt einer oder mehrerer Dateien konkateniert
werden soll.

Das folgende Beispiel liest im ersten Pipeline-Schritt die Datei
`dump1.dat` ein, entfernt ungültige Datensätze und gibt die Ausgabe nach
`stdout` aus. Der zweite Pipeline-Schritt liest diese Datensätze ein
(`-`) und konkateniert diese mit den Datensätzen aus der Datei
`dump2.dat`. Das Ergebnis wird in die Datei `out.dat` geschrieben.

```bash
$ pica cat -s dump1.dat | pica cat - dump2.dat -o out.dat
```

Der Dateiname `-` steht für die Standardeingabe (`stdin`). Wären die
zwei Argumente vertauscht (`pica cat dump2.dat -`), dann würden erst die
gültigen Datensätze aus der Datei `dump1.dat` und anschließend die
Datensätze aus dem ersten Pipeline-Schritt geschrieben.


### Hinzufügen von Datensätzen

Wenn eine Ausgabedatei bereits existiert, wird diese standardmäßig neu
angelegt und überschrieben. Soll das Verhalten dahingehend geändert
werden, dass an die bestehenden Dateien angehangen wird, kann dies mit
der `--append`-Option erreicht werden. Diese Option ändert das Verhalten
von `--output` und `--tee`. Die Option hat auf das Verhalten beim
Schreiben in die Standardausgabe keine Auswirkung.

Im folgenden Beispiel erzeugt der erste Befehl eine neue Datei
`gnd.dat`. Sollte die Datei bereits existieren, wird der Datei-Inhalt
überschrieben. Die darauffolgenden Kommandos hängen jeweils an das Ende
der Datei `gnd.dat` an.

```bash
$ pica cat Tp*.dat -o gnd.dat
$ pica cat --append Ts*.dat -o gnd.dat
$ pica cat --append Tu*.dat -o gnd.dat
...
```

### Abzweigen der Ausgabe

Gelegenlich kann es nützlich sein, die Ausgabe des `cat`-Kommandos in
eine Datei zu schreiben und gleichzeitig die Ausgabe an einen weiteren
Pipeline-Schritt weiterzureichen. Dies hat den Vorteil, dass zwei
CPU-Kerne gleichzeitig genutzt werden können. Mit der `--tee`-Option
lässt sich dieses Verhalten erzielen. Der Name der Option leitet sich
von dem T-Stück (engl. tee connector) ab, mit dem ein Klempner eine
Abzeigung in eine Leitung einbaut.

Im folgenden Beispiel werden alle `Tp*.dat` zusammengefügt und in eine
Datei `Tp.dat` geschrieben. Gleichzeitig werden alle Datensätze mit dem
[_filter_]-Kommando nach der Satzart `Tp2` im Feld `002@.0` gefiltert.

```bash
$ pica cat partitions/Tp*.dat --tee gnd_person.dat | \
      pica filter "002@.0 =^ 'Tp2'" -o Tp2.dat
```

### Komprimierte Ein- und Ausgabe

Das vorhergehende Beispiel zeigt auch, dass das `cat`-Kommando sowohl
unkomprimierte als auch komprimierte Dateien verarbeiten kann. Endet
eine Datei mit dem Suffix `.gz`, wird die Datei automatisch
dekomprimiert (als Eingabedatei) bzw. komprimiert (als Ausgabedatei).

Mittels der Option `-g`/`--gzip` erfolgt eine Komprimierung der Ausgabe
unabhängig von der Dateiendung:

```bash
$ pica cat --gzip ger.dat eng.dat -o eng_ger_compressed.dat
$ pica cat ger.dat eng.dat -o eng_ger.dat.gz
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
