# Filter

Eines der Kernelemente des Toolkits ist das Filtern von Datensätzen und
Feldern nach bestimmten Kriterien.

## PICA-Datenformat

Ein PICA-Datensatz (_record_) besteht aus einer nicht-leeren Liste von
Feldern (_field_), die über eine Feldnummer (_tag_) referenziert werden
und eine Liste von Unterfeldern enthalten. Optional können Felder einen
numerischen Zählwert (_occurrence_) enthalten, der je nach Kontext
unterschiedlich interpretiert werden kann. Unterfelder bestehen aus
einem Unterfeld-Code (_code_) und einem Wert (_value_). Eine umfassende
Darstellung des PICA-Datenformats findet sich in der [Einführung in die
Verarbeitung von PICA-Daten] von Jakob Voß.

[Einführung in die Verarbeitung von PICA-Daten]: https://pro4bib.github.io/pica


## Unterfelder

Ein Unterfeld besteht immer aus einem alpha-numerischen Unterfeld-Code
(`A-Z`, `a-z`, `0-9`) und einen Wert (String). Um zu testen, ob der Code
eines Unterfeldes einem bestimmten Wert entspricht, wird der
`?`-Operator (_exists_-Operator) verwendet: Der Ausdruck `0?` testet
bspw., ob Codes eines Unterfelds `0` ist, oder nicht. Soll getestet
werden, ob der Code eines Unterfelds einem Wert aus einer Liste
entspricht, werden alle Möglichkeiten in eckigen Klammern
zusammengefasst: Der Ausdruck `[adf]?` wird bei einem konkreten
Unterfeld wahr, wenn der Code entweder `a`, `d` oder `f` ist. Ist die
Auflistung von möglichen Codes sehr lang und lässt sich eine Spanne von
Codes finden, können diese mit einem `-` zusammengefasst werden
(`[a-z0-3]`). Sind alle Unterfeld-Codes gültig, wird dies mit `*?`
angezeigt.
