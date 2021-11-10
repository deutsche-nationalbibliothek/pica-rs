# pica.rs Anfänger-Tutorial
## Was ist pica.rs?

pica.rs ist ein Set von Kommandozeilen-Tools zur Arbeit mit PICA+-Bibliothekskatalog-Daten. Wenn Sie nicht wissen, was PICA-Daten sind, brauchen Sie diese Tools nicht. 😉 Große Datenabzüge bis hin zu Gesamtabzügen können schnell gefiltert werden und es können die Daten einzelner Felder und Unterfelder in CSV-Dateien exportiert werden, Häufigkeitsverteilungen des Inhalts einzelner Unterfeldern erfasst werden und vieles mehr.

## Wie funktioniert pica.rs?
Das Tool kann mit extrem großen Dateien umgehen, weil es diese sequentiell ausliest und prozessiert. Die Dateien werden nicht geöffnet und in den Arbeitsspeicher geladen, sondern ›häppchenweise‹ ausgewertet. Es ist deswegen kein Rechner mit besonders viel Arbeitsspeicher notwendig. Es empfiehlt sich aber, die Ausgangsdaten auf möglichst schnellen lokalen Laufwerken abzulegen. Netzlaufwerke sind weniger geeignet und verlangsamen das Tool unnötig.

pica.rs läuft unter Windows, Linux und Mac OS.

## Installation

Es ist möglich, die Quelldateien herunterzuladen und direkt auf dem eigenen Rechner von RUST zu einem lauffähigen Programm kompilieren zu lassen.

Für die gängigen Windows-, Apple- oder Linux-Systeme, stehen aber fertige Programmpakete unter (https://github.com/deutsche-nationalbibliothek/pica-rs/releases) zur Verfügung.

Entpacken Sie das Paket und legen es in einen beliebigen Ordner, z. B. unter Programme.

In der Konfigurationsdatei Ihres Terminals müssen Sie dann noch den Pfad angeben, in dem Sie das Programm abgelegt haben.

Beispiel: ZSH unter Linux oder MacOS

Die versteckte Datei `.zshrc` liegt üblicherweise im Homeverzeichnis des aktuellen Benutzers. Dort fügen Sie an beliebiger Stelle folgende Zeile hinzu:

```bash
export PATH="/PFADZUMPROGRAMM:$PATH"
```

wobei `PFADZUMPROGRAMM` natürlich durch Ihren tatsächlichen Pfad ersetzt werden muss.

Nach einem Neustart des Terminals sollte jetzt der neue Befehl `pica` zur Verfügung stehen. Mit `pica -V` können Sie testen, welche Version sie haben.

## Kommandozeile
pica.rs ist auch deswegen sehr schnell, weil es kein grafisches Interface hat. Man sollte deshalb einige Basics der Kommandozeilen (auch Terminal oder Shell genannt) des jeweiligen Betriebssystems kennen. Alle Befehle werden hier in der Fassung für gängige Linux-und Mac OS-Terminals gezeigt, abweichende Befehle der Windows Power Shell werden meistens in Klammern erwähnt.

Zum Testen steht unter [testdaten.dat] ein Testdatenpaket mit 1.000 zufällig ausgewählten Datensätzen aus dem Bestand der Deutschen Nationalbibliothek bereit. Der Test-Datensatz enthält sowohl Titeldaten als auch GND-Normdatensätze.

## Pipes
Um das Tool optimal nutzen zu können, sollten Sie verstehen, was __Pipes__ sind. Im Terminal wird die Ausgabe ausgeführter Programme oder Befehle üblicherweise in die sogenannte __Standardausgabe__ (`stdout`) geschrieben. Normalerweise ist das die Bildschirmausgabe des Terminals selbst. Wenn sie z. B. den Inhalt des aktuellen Ordners mit `ls` (Windows: `dir`) auslesen, wird eine Liste aller Dateien und Ordner direkt im Terminal ausgegeben.

Sie könnten diese Ausgabe aber auch umleiten: z.B. in eine Datei oder auf einen angeschlossenen Drucker etc.

Piping nennt man ein Verfahren, bei dem die Ausgabe eines Befehls direkt als Eingabe für einen weiteren Befehl verwendet wird. Wie Rohre (pipes) werden die Befehle aneinandergesteckt und die Daten fließen von einem Programm zum nächsten.

Dazu werden die Befehle mit einem senkrechten Strich verbunden: `|` Unter Linux und Windows ist dieser Strich normalerweise über die Tastenkombination `AltGr + <` zu erreichen, unter MacOS über `Alt + 7`.

Man könnte also z. B. die Ausgabe von `ls` bzw. `dir` an einen Befehl weiterleiten, der die Anzahl der ausgegeben Zeilen zählt. Dieser Befehl heißt `wc -l` (von word count -lines). Das korrekte Piping geht so:

```bash
ls | wc -l
```

Die Ausgabe von Word Count lässt sich wieder weiterleiten, z.B. in eine Datei:

```bash
ls | wc -l > ordnerinhalt.txt
```

Der `>`-Operator leitet den Inhalt in eine Datei weiter und ist eine Art Sonderfall des Pipings, der nur für das Schreiben in Dateien gilt.

Man könnte die Ausgabe mit einer weiteren Pipe auch an noch einen weiteren Befehl übergeben.

Mit Pipes lassen sich die einzelnen pica.rs-Tools (select, filter, frequency usw.) miteinander verknüpfen. Die Ausgabe des einen Tools kann entweder zum nächsten Tool, in eine Datei oder einfach auf den Bildschirm geleitet werden. Alle Tools (außer cat und ?) schreiben immer in die Standardausgabe. Will man die Ausgabe anders erhalten, muss man das dem Befehl mitteilen.

## Los geht’s
Navigieren Sie im Terminal zu dem Ordner, in dem das Testdatenpaket liegt. Wir gehen davon aus, dass Sie im Hauptverzeichnis Ihres aktuellen Benutzers (unter Linux und Mac OS über das Kürzel `~` zu erreichen) im Verzeichnis `pica-test` arbeiten. Das Testdatenpaket heißt `testdaten.dat`.

```bash
cd ~/pica-test
```

Überprüfen Sie, ob das Testdatenpaket vorhanden ist.

```bash
ls (unter Windows: dir)
```

Sie sehen etwas wie:

```bash
total 1872
drwxr-xr-x   3 testuser  staff    96B  9 Nov 14:24 .
drwxr-xr-x+ 76 testuser  staff   2,4K  9 Nov 14:25 ..
-rw-r--r--@  1 testuser  staff   935K 14 Sep 18:30 testdaten.dat
```

## print
Wir beginnen mit mit __pica print__. Dieses Tool formatiert die unleserlichen PICA+-Daten zu gut lesbaren Datensätzen. Mit dem Befehl lassen sich die teilweise unübersichtlichen Daten gut überblicken. Wir wollen nur einen Datensatz aus den Testdaten auf dem Bildschirm ausgeben.

```bash
pica print -l 1 testdaten.dat
```

Die Option `-l` steht für Limit und begrenzt die Ausgabe auf einen Datensatz. Die folgende Ziffer gibt die Anzahl der auszugebenden Datensätze an.

Wir können die Ausgabe auch in eine Datei schreiben:

```bash
pica print -l 1 testdaten.dat -o testdatensatz.txt
```

Wenn Sie nur einen Dateinamen angeben, wird die Datei im aktuellen Verzeichnis abgelegt. Wollen sie in ein anderes Verzeichnis schreiben, müssen sie den kompletten Pfad dorthin angeben.

Im Folgenden gehen wir davon aus, dass Sie grundlegend mit der Struktur von Pica-Daten vertraut sind, also z. B. Feldern und Unterfeldern, Satzarten, Codes etc.

## filter

Mit __filter__ können Teilmengen aus einem Daten-Dump nach einem bestimmten Selektionskriterium gebildet werden. __filter__ gibt grundsätzlich den ganzen Datensatz aus, wenn die angegebenen Filterkriterien erfüllt sind.

Wir wissen, dass in unseren Testdaten jeweils 100 Datensätze der unterschiedlichen Satzarten enthalten sind. Wir wollen alle Oa-Sätze herausfiltern und den ersten davon mit `print` ausgeben.

```bash
pica filter -s "002@.0 == 'Oa'" testdaten.dat | pica print -l 1
```

Das Ergebnis könnte man auch wieder in eine Datei schreiben:

```bash
pica filter -s "002@.0 == 'Oa'" testdaten.dat -o oa-test.dat
```

Achtung: Dateien werden ohne Rückfrage überschrieben und werden nicht im Papierkorb gesichert. Gewöhnen Sie sich am besten an, in ein eigenes Ausgabeverzeichnis zu schreiben oder fügen Sie das aktuelle Datum an den Ausgabedateinamen an, damit sie nicht ausversehen eine ältere Datei überschreiben.

### Filter-Ausdrücke

Der Filterausdruck in den doppelten Anführungszeichen ist das mächtigste Werkzeug von pica.rs. Mehrere Ausdrücke können zu komplexen Suchfiltern kombiniert werden.

Jeder Filterausdruck besteht immer aus einem Feld wie `002@`, einem Unterfeldfilter wie `.0`, einem Operator, der angibt, wie der Inhalt des Feldes gefiltert werden soll, wie z. B. `==` und einem Wert, mit dem das Feld verglichen werden soll.

### Felder

Felder können in der einfachsten Form direkt benannt werden: `002@`

Felder können auch nummerierte Okkurrenzen haben wie `/01`. Okkurrenzen lassen sich nach ihrem Wert filtern oder alle Okkurrenzen können mit `/*` durchsucht werden. Bereiche von Okkurrenzen können ebenfalls eingegrenzt werden: `047A/01-03`

### Unterfelder

Unterfelder werden mit einem Punkt und ohne Dollarzeichen angehängt: `002@.9` meint Unterfeld `$9` von Feld `002@`.

Um z. B. Unterfeld `9` aller Okkurrenzen von Feld `041A` zu filtern, müsste der Feldausdruck lauten: `041A/*.9`.

### Operatoren

Werte können über folgende Vergleichsoperatoren gesucht werden.

- gleich `==` 
- strict equal `===`
- ungleich `!=`
- beginnt mit Prefix `=^`
- endet mit Suffix `=$`
- entspricht regulärem Ausdruck `=~`
- enthalten in `in`
- nicht enthalten in `not in`
- Feld existiert `?`

Die Operatoren können in runden Klammern gruppiert und mit den boolschen Operatoren UND `&&` sowie ODER `||` verbunden werden.

#TODO Beispiele und Erklärung aller Operatoren

#### == und ===

Der ==-Operator prüft, ob es ein Unterfeld gibt, dass einem Wert entspricht. `pica filter "012A.a == 'abc'"` liest sich wie folgt: Es existiert ein Feld `012A` mit *einem* Unterfeld `a` das gleich `abc` ist. Es könnten noch weitere Unterfelder `a` existieren, die nicht `abc` sind.

Im Gegensatz dazu stellt der ===-Operator sicher, dass *alle* Unterfelder `a` gleich `abc` sind. `pica filter "012A.a == 'abc'"` liest sich wie folgt: Es existiert ein Feld `012A` bei dem *alle* Unterfelder `a` gleich `abc` sind.

Bei beiden Varianten ist es nicht ausgeschlossen, dass es noch ein weiteres Feld `012A` gibt, dass kein Unterfeld `a` enthält.

#### !=

Das Gegenstück zu `==`. Prüft, ob ein Unterfeld nicht einem Wert entspricht.

#### =^

Prüft, ob ein Unterfeld mit einem bestimmten Prefix beginnt.

#### =$

Prüft, ob ein Unterfeld mit einem bestimmten Suffix endet.

#### =~

Prüft ob ein Feld einem regulären Ausdruck entspricht. Die Auswertung dieses Operators benötigt die meiste Rechenkapazität. Er sollte deshalb nur dann verwendet werden, wenn er wirklich absolut notwendig ist. Es ist z. B. schneller, nach einer Kombination von `=^` und `=$` zu suchen als nach einem regulären Ausdruck.

Tipp: ein empfehlenswertes Tool, um reguläre Ausdrücke zu schreiben und zu testen, ist (regex101.com)[https://regex101.com].

#### in und not in

Prüft, ob ein Unterfeld in einer Liste von Werten enthalten ist. Die Werte stehen in eckigen Klammern und sind durch Kommas getrennt. `not in` ist die Umkehrung dazu und prüft, ob Unterfeld nicht in der Werteliste enthalten ist.

Beispiel:

```bash
pica filter -s "0100.a in ['ger', 'eng']" testdaten.dat
```
#### ?

Prüft. ob ein Feld oder ein Unterfeld überhaupt existiert.

```bash
pica filter -s "012A/00?" testdaten.dat
pica filter -s "002@.0?" testdaten.dat
pica filter -s "002@{0?}" testdaten.dat
```
### mehrere Felder adressieren

Es kommt öfters vor, dass sich ein Wert vom gleichen Typ in unterschiedlichen Feldern befindet. Z. B. befindet sich im Feld `028A.9` die "Person, Familie - 1. geistiger Schöpfer" und im Feld `029A.9` "Person, Familie - weitere geistige Schöpfer". Um Datensätze zu filtern, die entweder einen 1. geistigen Schöpfer oder einen weiteren geistigen Schöpfer haben, könnte man schreiben:

```bash
pica filter "028A.9? || 029A.9?" testdaten.dat
```

Der Ausdruck lässt sich vereinfachen zu:

```bash
pica filter "02[89]A.9?" testdaten.dat
```

An jeder Position in einem Feld kann eine Liste der gültigen Werte angegeben werden. Es wird dann jede mögliche Kombination ausprobiert, um einen Match zu finden. Bsp. `0[12][34]A` führt zu der Liste `013A`, `014A`, `023A` und `024A`.

### mehrere Unterfelder adressieren

So ähnlich können auch mehrere Unterfelder adressiert werden. Beispiel: Im Feld `045E` befindet sich die Sachgruppe der Deutschen Nationabibliografie. Im Unterfeld `$e` die Hauptsachgruppe (HSG) und im Feld `$f` die Nebensachgruppen (NSG). Ist man an allen Datensätzen interessiert, die zur HSG 100 oder zur NSG 100 gehören, könnte man folgenden Filter schreiben:

```bash
pica filter "045E.e == '100' || 045E.f == '100'" testdaten.dat
```

Der Ausdruck lässt sich verkürzen zu:

```bash
pica filter "045E.[ef] == '100'" testdaten.dat
```

Beide Verfahren sind kombinierbar: `0[12]3[AB].[xyz]` ist ein gültiger Ausdruck.

## Select

Mit __select__ können die Werte einzelner Unterfelder in eine CSV-Datei exportiert werden. Dabei können mehrere Unterfelder kombiniert werden. Man kann aus riesigen Datenbeständen exakt die Daten extrahieren, die man für weitere Datenanalyse benötigt.

Der Selektionsausdruck enthält eine durch Kommas getrennte Liste von Unterfeldern, die ausgelesen werden sollen, z. B.:

```bash
pica select "002@.0, 003@.0" testdaten.dat -o test-select.csv
```

Das Ergebnis ist eine CSV-Datei mit zwei Spalten, in diesem Beispiel einer Spalte für die Satzart und einer Spalte für die IDN.

Wenn Felder mehrere Unterfelder haben, können diese in einer Liste in geschweiften Klammer an die Feldbezeichnung angehängt werden.

```bash
pica select "002@.0, 003@.0, 021A{a,h}" testdaten.dat -o test-select.csv
```

In die Selektionsausdrücke können auch Filterausdrücke eingebaut werden. Dazu muss die erste Position der Liste in den geschweiften Klammern mit einem Filterausdruck belegt werden.

```bash
pica select "003@.0, 028A{4 == 'aut',9,d,a}" testdaten.dat -o test-select.csv
```

In diesem Beispiel werden die Angaben zu den beteiligten Personen aus Feld 028A nur übernommen, wenn Unterfeld 4 den Wert `aut` enthält, die Person also Autor\*in ist und nicht etwa Herausgeber\*in.

Für diese Filterausdrücke gelten dieselben Regeln wie für Filterausdrücke im filter-Tool, die oben beschrieben wurden.

Wenn Felder wiederholbar sind (z. B. bei Schlagworten), wird pro Wiederholung eine neue Zeile in die CSV ausgegeben. Die ausgegebene CSV-Datei kann also mehr Zeilen enthalten, als Datensätze in den Ausgangsdaten waren. Es empfiehlt sich deshalb einen eindeutigen Identifikator mitzuselektieren, damit die wiederholten Felddaten von neuen Datensätzen unterschieden werden können.

Es können auch Spaltennamen für die CSV-Ausgabe angegeben werden mit der Option -H. Wichtig: die Anzahl Spaltennamen muss der Anzahl der selektierten Unterfelder entsprechen.

```bash
pica select -H "idn, autor-idn, autor-vorname, autor-nachname" "003@.0, 028A{4 == 'aut',9,d,a}" testdaten.dat -o test-select.csv
```

## Warum zwei Filtermöglichkeiten?
#TODO
Die doppelte Filtermöglichkeit einmal mit dem Filter-Tool und einmal im select-Tool verwirrt auf den ersten Blick etwas. `filter` prüft eine oder mehrere Felder oder Unterfelder auf Bedingungen und gibt den gesamten Datensatz aus, wenn die Bedingung wahr ist. `select` prüft ebenfalls auf Bedingungen und selektiert dann die benötigten Teildaten.

Man könnte auch sagen: mit `filter` wird die Zahl der Datensätze reduziert und mit `select` werden die einzelnen Datenpunkte ausgelesen. 

## Arbeit mit großen Datenabzügen

pica.rs parst immer den kompletten Datenbestand, auch wenn man nur wenige Ergebnisse erwartet. Deshalb ist es manchmal sinnvoll, die Ausgangsdatei in kleinere Dateien zu teilen, die dann viel schneller verarbeitet werden können.

In unseren Testdaten haben wir Titeldaten und Normdaten zusammen. Es könnte z.B. sinnvoll sein, die Normdaten zu extrahieren, wenn man keine Titeldaten braucht oder nur eine bestimmte Satzart zu extrahieren, wenn man nur innerhalb dieser Satzart suchen will.

#TODO
- partition
- slice
- split