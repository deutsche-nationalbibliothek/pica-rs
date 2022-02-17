# cat

Das `cat` Kommando liest Datensätze direkt von der Standardeingabe (`stdin`) oder aus Dateien ein
und fügt diese zu einem Stream zusammen, der entweder in eine Datei oder in die Standardausgabe
(`stdout`) geschrieben wird.

> Hinweis: Das Kommando befindet sich im Beta-Status und wird gerade intensiv getestet, bevor
> es als stabil freigegeben wird. Änderungen am _command-line interface_ (CLI) sowie das
> Auftreten kleinerer Fehler ist möglich.

## Beschreibung

Der wichtigste Anwendungsfall für das Programm `cat` ist in Kombination mit den Kommandos
[_partition_] oder [_split_], da es die Funktion (teil-)rückgängig machen kann. Häufig macht es Sinn,
eine große Datei in viele kleinere Datein anhand eines Kriteriums zu teilen. Sollen die Datensätze
wieder zu einer Datei zusammengefügt werden, kann das `cat`-Kommando genutzt werden.

<img src="cat1.png" class="center" />

Das folgende Beispiel fügt die zwei Partitionen `ger.dat` und `eng.dat` zu einer Datei zusammen.

```bash
$ pica cat ger.dat eng.dat -o ger_eng.dat
```

### Überspringen ungültiger Datensätze

Der obige Befehl geht davon aus, dass die zwei Partition ausschließlich gültige Datensätze
enthalten. Gültig in diesem Zusammenhang bedeutet, dass es sich um valide Datensätze im format
PICA+ handelt und nicht ob ein Datensatz einem bestimmten Regelwerk entspricht.

Das Ausschließen von ungültigen Datensätzen wird mit der Option `--skip-invalid` oder `-s` erreicht:

```bash
$ pica cat --skip-invalid DUMP.dat.gz -o dump_valid.dat
$ pica cat -s DUMP.dat.gz --output dump_valid.dat.gz
```

### Komprimierte Ein- und Ausgabe

Die beiden Befehlen veranschaulichen auch, dass das `cat`-Kommando sowohl unkompromierte als auch
komprimierte Dateien verarbeiten kann. Endet eine Datei mit dem Suffix `.gz` wird die Datei automatisch
dekompromiert (als Eingabedatei) bzw. komprimierte (als Ausgabedatei). Soll eine Komprimierung in der
Ausgabe unabhängig von der Dateiendung erfolgen, kann dies mit der Option `--gzip` erreicht werden:

```bash
$ pica cat --gzip ger.dat eng.dat -o eng_ger_compressed.dat
$ pica cat ger.dat eng.dat -o eng_ger.dat.gz
```

### Lesen von der Standardeingabe

Das Kommando kann auch direkt von der Standardeingabe (`stdin`) lesen. Das ist dann hilfreich, wenn
diedie Ausgabe aus einem vorhergehenden Pipeline-Schritt mit dem Inhalt einer oder mehrerer Dateien
konkateniert werden soll. Das folgende Beispiel liest im ersten Pipeline-Schritt die Datei `eng.dat`
ein, entfernt ungültige Datensätze und gibt die Ausgabe nach `stdout` aus. Der zweite Pipeline-Schritt
liest diese Datensätze ein (`-`) und konkateniert diese mit den Datensätzen aus der Datei `ger.dat`.
Das Ergebnis wird in die Datei `eng_ger.dat` geschrieben.

```bash
$ pica cat -s eng.dat | pica cat - ger.dat -o eng_ger.dat
```

Der Dateiname `-` steht für die Standardeingabe (`stdin`). Wären die zwei Argumente vertauscht
(`pica cat ger.dat -`), dann würden erst die Datensätze aus der Datei `ger.dat` und anschließend die
Datensätze aus dem ersten Pipeline-Schritt geschrieben.

### Abzweigen der Ausgabe

Manchmal kann es nützlich sein, die Ausgabe des `cat`-Kommandos in eine Datei zu schreiben und
gleichzeitig die Ausgabe an einen weiteren Pipeline-Schritt weiterzureichen. Diese hat den Vorteil,
dass zwei CPU-Kerne gleichzeitig genutzt werden können. Mit der `--tee`-Option lässt sich dieses
Verhalten erzielen. Der Name der Option leitet sich von dem T-Stück (engl. tee connector) ab, mit
dem ein Klemptner eine Abzeigung in eine Leitung einbaut. Das Verhalten der Option veranschaulicht
das folgende Bild:

<img src="cat2.png" class="center" style="width: 75%" />

Im folgenden Beispiel werden alle `Tp*.dat` zusammengefügt und in eine Datei `Tp.dat` geschrieben.
Gleichzeitig werden alle Datensätze mit dem [_filter_]-Kommando danach gefiltert, ob die Satzart
(Feld `002@.0`) gleich dem Wert `Tp2`[^2] ist. Das Ergebnis wird in eine zweite Datei `Tp2.dat`
geschrieben.

```bash
$ pica cat partitions/Tp*.dat --tee gnd_person.dat | \
      pica filter "002@.0 =^ 'Tp2'" -o gnd_person_level2.dat
```

### Hinzufügen von Datensätzen

Wenn eine Ausgabedatei bereits existiert, wird diese standardmäßig neu angeleg und überschrieben.
Soll das Verhalten so geändert werden, dass an die bestehenden Dateien angehangen wird, kann dies
mit der `--append`-Option erreicht werden. Diese Option ändert das Verhalten von `--output` und
`--tee`. Die Option hat auf das Verhalten beim Schreiben in die Standardausgabe keine Auswirkung.


Im folgenden Beispiel erzeugt der erste Befehl eine neue Datei `gnd.dat`. Sollte die Datei bereits
existieren, wird der Datei-Inhalt überschrieben. Die folgenden Kommandos hängen jeweils an das Ende
der Datei `gnd.dat` an.

```bash
$ pica cat Tp*.dat -o gnd.dat
$ pica cat --apend Ts*.dat -o gnd.dat
$ pica cat --apend Tu*.dat -o gnd.dat
...
```

[_filter_]: filter.md
[_partition_]: partition.md
[_split_]: split.md

