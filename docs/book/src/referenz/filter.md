# Filter

## Operatoren

| Operator    | Beschreibung                                                                                  | Beispiel                           |
| ----------- | --------------------------------------------------------------------------------------------- | ---------------------------------- |
| `==` / `!=` | Überprüft, ob der Wert eines Unterfelds gleich bzw. ungleich ist                              | `002@.0 == 'Tp', `008A.a != 's'`   |
| `=^` / `!^` | Überprüft, ob der Wert eines Unterfelds mit dem angegebenen Prefix beginnt bzw. nicht beginnt | `002@.0 =^ 'Tp'`, `002@.0 !^ 'Tu'` |
| `=$` / `!$` | Überprüft, ob der Wert eines Unterfelds mit dem angegebenen Suffix endet bzw. nicht endet     | `002@.0 =^ '1'`, `002@.0 !^ 'z'`   |

