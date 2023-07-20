# print

![stability-badge](https://img.shields.io/badge/stability-stable-green?style=flat-square)

Das `print`-Kommando gibt Datensätze in einer menschenlesbaren Form aus.


## Beschreibung

Mithilfe des `print`-Kommandos können Datensätze in einer
menschenlesbaren Form auf dem Terminal ausgegeben oder in eine Datei
geschrieben werden. Das Format ist an die Darstellung in der WinIBW
angelehnt: Felder werden zeilenweise ausgegeben; zuerst wird das Feld
(`003@`), dann optional die Okkurrenz (`/01`), und schließlich die Liste
von Unterfeldern ausgegeben. Dem Unterfeld-Code wird ein Dollarzeichen
vorangestellt. Die Unterfeldwerte werden genau so ausgegen, wie sie im
Datensatz vorhanden sind; es findet kein [_Escaping_] von Sonderzeichen
statt. Einzelne Datensätze werden durch eine Leerzeile voneinander
getrennt.

Der folgende Befehl gibt den ersten Datensatz aus:

```console
$ pica print -s -l1 DUMP.dat.gz
001A $0 1250:01-07-88
001B $0 9999:15-04-22 $t 15:15:00.000
001D $0 0292:01-08-19
001U $0 utf8
001X $0 0
002@ $0 Tpz
003@ $0 118540238
003U $a http://d-nb.info/gnd/118540238 $z http://d-nb.info/gnd/185808069 $z http://d-nb.info/gnd/185848826 $z http://d-nb.info/gnd/101488358X $z http://d-nb.info/gnd/1014927390 $z http://d-nb.info/gnd/1022736213 $z http://d-nb.info/gnd/1095607278 $z http://d-nb.info/gnd/1131918517
004B $a piz
006Y $S isni $0 0000 0001 2099 9104
006Y $S wikidata $0 Q5879
...
```

## Optionen

* `-s`, `--skip-invalid` — überspringt jene Zeilen aus der Eingabe, die
  nicht dekodiert werden konnten.
* `-l`, `--limit` `<n>` — Eingrenzung der Ausgabe auf die ersten _n_
  Datensätze.
* `--translit` `<nf>` — Ausgabe wird in die angegebene Normalform
  transliteriert. Mögliche Werte: `nfd`, `nfkd`, `nfc` und `nfkc`.
* `--color` `<color>` — Auswahl der Farbeinstellung (mögliche Werte:
  `auto`, `always`, `ansi` und `never`). Standardmäßig ist der Wert
  `auto` gesetzt. Die Einstellung `always` versucht möglichst immer eine
  Farbausgabe. Ähnlich funktioniert die Einstellung `ansi`, nur mit dem
  Unterschied, dass ausschließlich ANSI-Farbcodes für die Darstellung
  von Farben verwendet werden. Mit der Einstellung `auto` wird eine
  Farbausgabe angestrebt, aber nicht erzwungen. Bspw. wird bei `auto`
  keine Farbausgabe durchgeführt, wenn der Terminal dies nicht
  unterstütz oder die Umgebungsvariable `NO_COLOR` definiert ist.
  Schließlich wird mit der Einstellung `never` die Farbausgabe
  deaktiviert.
* `-o`, `--output` — Angabe, in welche Datei die Ausgabe geschrieben
  werden soll. Standardmäßig wird die Ausgabe in die Standardausgabe
  `stdout` geschrieben.


## Konfiguration

<!-- TODO: Link zum allgemeinen Kapitel über die Konfigurationsdatei -->

Die Option zum Ignorieren invalider Datensätze sowie Farbeinstellungen
lassen sich in der `Pica.toml` konfigurieren:

```toml
[print]
tag-color = { color = "red", bold = true, intense = true }
occurrence-color = { color = "blue", underline = true }
code-color = { color = "165,42,42", italic = false }
value-color = { color = "95", dimmed = true }

skip-invalid = true
```

Die Werte der Kommandozeilen-Optionen haben Vorrang vor den Werten aus
der Konfiguration.

Folgende Farbeinstellung können getroffen werden:

* `tag-color` — Farbe des PICA-Tags (bspw. `041A`)
* `occurrence-color` — Farbe der PICA-Okkurrenz (bspw. `/01`)
* `code-color` — Farbe eines Unterfeld-Codes  (bspw. `$a`)
* `value-color` — Farbe eines Unterfeld-Werts  (bspw. `Goethe`)

Jede Farbeinstellung kann folgende Einstellungen vornehmen:

* `color` — Festlegen der Vordergrundfarbe. Folgende Werte sind erlaubt:
  `black`, `blue`, `green`, `red`, `cyan`, `magenta`, `yellow`, `white`,
  ein [_Ansi256 Farbcode_] oder ein RGB-Farbwert.
* `bold` — Festlegen, ob der Text fett gedruckt werden soll (`true` /
  `false`).
* `italic` — Festlegen, ob der Text kursiv gedruckt werden soll (`true`
  / `false`).
* `underline` — Festlegen, ob der Text unterstrichen werden soll (`true`
  / `false`).
* `intense` — Festlegen, ob der Text intensiv ausgegeben werden soll
  (`true` / `false`).
* `dimmed` — Festlegen, ob der Text gedimmt ausgegeben werden soll
  (`true` / `false`).

Die tatsächliche Farbausgabe ist vom Betriebssystem und der
Terminaleinstellung abhängig.


## Beispiele

### Transliteration der Ausgabe

Standardmäßig werden die Unterfeldwerte so ausgabeben wie sie im
Datensatz vorkommen. Mit der Option `--translit` werden die Werte in die
angegebene [_Unicode-Normalform_] transliteriert.

```console
$ pica print -s -l1 --translit nfc DUMP.dat.gz
...
028@ $d Yohan Wolfgang $a Gete
028@ $d Yôhân Wôlfgang $c fôn $a Gete
028@ $d Yôhan Wolfgang $a Gête
028@ $d Yohann Volfqanq $a Gete
028@ $d Yogann Volʹfgang $a Gete
...
028@ $T 01 $U Cyrl $L uzb $d Йоҳанн Волфганг $a Гёте
028@ $T 01 $U Hans $P 歌德 $5 DE-576
028@ $T 01 $U Hant $P 約翰・沃爾夫岡・馮・歌德 $5 DE-576
028@ $T 01 $U Hans $P 约翰・沃尔夫冈・冯・歌德 $5 DE-576
028@ $T 01 $U Jpan $d ヨハン・ヴォルフガング・フォン $a ゲーテ $5 DE-576
028@ $T 01 $U Hebr $d יוהן וולפגנג פון $a גתה
028@ $T 01 $U Hans $P 歌德
028@ $T 01 $U Jpan $d ヨハン・ヴォルフガング・フォン $a ゲーテ
028@ $T 01 $U Geor $d იოჰან ვოლფგანგ ფონ $a გოეთე
028A $d Johann Wolfgang $c von $a Goethe
...
```


[_Ansi256 Farbcode_]: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797#256-colors
[_Escaping_]: https://de.wikipedia.org/wiki/Escape-Sequenz
[_Unicode-Normalform_]: https://de.wikipedia.org/wiki/Normalisierung_(Unicode)
