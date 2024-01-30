# Filter

## Operatoren

| Operator       | Beschreibung                                                   | Beispiele                                     |
| -------------- | -------------------------------------------------------------- | --------------------------------------------- |
| `==`, `!=`     | Testet auf Gleichheit bzw. Ungleichheit                         | `002@.0 == 'Tp'`, `008A.a != 's'`             |
| `=^`, `!^`     | Testet, ob ein Wert mit einem Prefix beginnt bzw. nicht beginnt | `002@.0 =^ 'Tp'`, `002@.0 !^ 'Tu'`            |
| `=$`, `!$`     | Testet, ob ein Wert mit einem Suffix endet bzw. nicht endet     | `002@.0 =^ '1'`, `002@.0 !^ 'z'`              |
| `=~`, `!~`     | Testet, ob ein Wert einem regulären Audruck entspricht bzw. nicht entspricht | `002@.0 =~ '^T[pfgpsu][1-3z]$'`  |
| `=?`           | Testet, ob ein Wert eine Zeichenkette enthält                   | `021A.a =? 'COVID-19'`                        |
| `in`, `not in` | Testet, ob ein Wert in einer Liste enthalten ist bzw. nicht enthalten ist | `002@.0 in ['Tp1', 'Ts1']`          |
| `=*`           | Testet, ob ein Wert eine Zeichenkette ähnlich ist[^1]           | `028A.d =* 'Heike'`                           |
| `#`            | Testet, ob die Anzahl der Felder/Unterfelder einer bestimmten Anzahl entspricht^[2] | `#203@/* > 10`                | `


[^1]: Der zulässige Schwellenwertkann über die Option `--strsim-threshold` gesetzt werden.
[^2]: Beim `#`-Operator konnen die Vergleichsoperatren `>`, `>=`, `==`, `!=`, `<` und `<=` verwendet werden. Der Vergleichswert muss 
  immer einen positive Ganzzahl sein und ohne Anführungszeichen angegeben werden.
