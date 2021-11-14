# Record Matcher

> **<sup>Syntax</sup>**\
> _RecordMatcher_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_RecordMatcherSingleton_]\
> &nbsp;&nbsp; | [_RecordMatcherGroup_]\
> &nbsp;&nbsp; | [_RecordMatcherNot_]\
> &nbsp;&nbsp; | [_RecordMatcherComposite_]\
> &nbsp;&nbsp; | [_RecordMatcherTrue_]

## Record Matcher Singleton

> **<sup>Syntax</sup>**\
> _RecordMatcherSingleton_ : [_FieldMatcher_]

## Record Matcher Composite

> **<sup>Syntax</sup>**\
> _RecordMatcherComposite_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; RecordMatcherCompositeOr\
> &nbsp;&nbsp; | RecordMatcherCompositeAnd\
>\
> _RecordMatcherCompositeOr_ : OrOperand `||` OrOperand\
>\
> _OrOperand_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_RecordMatcherGroup_]\
> &nbsp;&nbsp; | _RecordMatcherCompositeAnd_\
> &nbsp;&nbsp; | [_RecordMatcherSingleton_]\
> &nbsp;&nbsp; | [_RecordMatcherNot_]
>\
> _RecordMatcherCompositeAnd_ : AndOperand `&&` AndOperand\
>\
> _AndOperand_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_RecordMatcherGroup_]\
> &nbsp;&nbsp; | [_RecordMatcherSingleton_]\
> &nbsp;&nbsp; | [_RecordMatcherNot_]

## Record Matcher Group

> **<sup>Syntax</sup>**\
> _RecordMatcherGroup_ : `(` _RecordMatcherGroupInner_ `)`\
>\
> _RecordMatcherGroupInner_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_RecordMatcherComposite_]\
> &nbsp;&nbsp; | [_RecordMatcherSingleton_]\
> &nbsp;&nbsp; | [_RecordMatcherNot_]\
> &nbsp;&nbsp; | [_RecordMatcherGroup_]

## Record Matcher Not

> **<sup>Syntax</sup>**\
> _RecordMatcherNot_ : `!` _RecordMatcherNotInner_\
>\
> _RecordMatcherNotInner_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_RecordMatcherComposite_]\
> &nbsp;&nbsp; | [_RecordMatcherSingleton_]\
> &nbsp;&nbsp; | [_RecordMatcherNot_]\
> &nbsp;&nbsp; | [_RecordMatcherGroup_]

## Record Matcher True

TBD

# Field Matcher

> **<sup>Syntax</sup>**\
> _FieldMatcher_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_FieldMatcherSubfield_]\
> &nbsp;&nbsp; | [_FieldMatcherExists_]

## Field Matcher Subfield

> **<sup>Syntax</sup>**\
> _FieldMatcherSubfield_ : [_TagMatcher_] [_OccurrenceMatcher_] (_DotExpr_ | _BracketExpr_)\
>\
> _DotExpr_ : `.` [_SubfieldListMatcherSingleton_]\
> _BracketExpr_: `{` [_SubfieldListMatcher_] `}`

## Field Matcher Exists

> **<sup>Syntax</sup>**\
> _FieldMatcherExists_ : [_TagMatcher_] [_OccurrenceMatcher_] `?`

# Tag Matcher

> **<sup>Syntax</sup>**\
> _TagMatcher_ :\
> &nbsp;&nbsp; `/` ( _TageMatcherSome_ | _TagMatcherPattern_ )\
>\
> _TagMatcherSome_ :\
> &nbsp;&nbsp; [_Tag_]\
>\
> _TagMatcherPattern_ :\
> &nbsp;&nbsp; _Digit0_ _Digit1_ _Digit2_ _Digit3_\
>\
> _Digit0_ : ( [0-2] | `[` [0-2]+ `]`)\
> _Digit1_ : ( [0-9] | `[` [0-9]+ `]`)\
> _Digit2_ : ( [0-9] | `[` [0-9]+ `]`)\
> _Digit3_ : ( ([A-Z] | '@') | `[` ([A-Z] | '@')+ `]`)

# Occurrence Matcher

> **<sup>Syntax</sup>**\
> _OccurrenceMatcher_ : `/` (\
> &nbsp;&nbsp; &nbsp;&nbsp; _OccurrenceMatcherRange_\
> &nbsp;&nbsp; | _OccurrenceMatcherSome_\
> &nbsp;&nbsp; | _OccurrenceMatcherNone_\
> &nbsp;&nbsp; | _OccurrenceMatcherAny_\
> )\
>\
> _OccurrenceMatcherRange_ : [_OccurrenceDigits_] `-` [_OccurrenceDigits_]\
> _OccurrenceMatcherSome_ : [_OccurrenceDigits_]\
> _OccurrenceMatcherNone_ : `00`\
> _OccurrenceMatcherAny_ : `*`

# Subfield List Matcher

> **<sup>Syntax</sup>**\
> _SubfieldListMatcher_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldListMatcherGroup_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherNot_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherComposite_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherSingleton_]

## Subfield List Matcher Singleton

> **<sup>Syntax</sup>**\
> _SubfieldListMatcherSingleton_ : [_SubfieldMatcher_]\

## Subfield List Matcher Group

> **<sup>Syntax</sup>**\
> _SubfieldListMatcherGroup_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; `(` _SubfieldListMatcherGroupInner_ `)`\
>\
> _SubfieldListMatcherGroupInner_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldListMatcherComposite_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherSingleton_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherNot_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherGroup_]

## Subfield List Matcher Not

> **<sup>Syntax</sup>**\
> _SubfieldListMatcherNot_ : `!` _SubfieldListMatcherNotInner_\
>\
> _SubfieldListMatcherNotInner_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldListMatcherGroup_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherSingleton_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherNot_]

## Subfield List Matcher Composite

> **<sup>Syntax</sup>**\
> _SubfieldListMatcherComposite_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; SubfieldListMatcherCompositeOr\
> &nbsp;&nbsp; | SubfieldListMatcherCompositeAnd\
>\
> _SubfieldListMatcherCompositeOr_ : OrOperand `||` OrOperand\
>\
> _OrOperand_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldListMatcherGroup_]\
> &nbsp;&nbsp; | _SubfieldListMatcherCompositeAnd_\
> &nbsp;&nbsp; | [_SubfieldListMatcherSingleton_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherNot_]\
>\
> _SubfieldListMatcherCompositeAnd_ : AndOperand `&&` AndOperand\
>\
> _AndOperand_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldListMatcherGroup_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherSingleton_]\
> &nbsp;&nbsp; | [_SubfieldListMatcherNot_]


# Subfield Matcher

> **<sup>Syntax</sup>**\
> _SubfieldMatcher_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldMatcherComparison_]\
> &nbsp;&nbsp; | [_SubfieldMatcherRegex_]\
> &nbsp;&nbsp; | [_SubfieldMatcherIn_]\
> &nbsp;&nbsp; | [_SubfieldMatcherExists_]

## Subfield Matcher Comparison

> **<sup>Syntax</sup>**\
> _SubfieldMatcherComparison_ : [_SubfieldCodes_] [_ComparisonOp_] [_StringLiteral_]

## Subfield Matcher Regex

> **<sup>Syntax</sup>**\
> _SubfieldMatcherRegex_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldCodes_] `=~` [_StringLiteral_]\
> &nbsp;&nbsp; | [_SubfieldCodes_] `!~` [_StringLiteral_]


## Subfield Matcher In

> **<sup>Syntax</sup>**\
> _SubfieldMatcherIn_ :\
> &nbsp;&nbsp; &nbsp;&nbsp; [_SubfieldCodes_] `in` [_StringLiteralList_]\
> &nbsp;&nbsp; | [_SubfieldCodes_] `not in` [_StringLiteralList_]


## Subfield Matcher Exists

> **<sup>Syntax</sup>**\
> _SubfieldMatcherExists_ : [_SubfieldCodes_] `?`

# Comparison Operators

> **<sup>Syntax</sup>**\
> _ComparisonOp_ : `==` | `!=` | `=^` | `=$`

# Boolean Operators

> **<sup>Syntax</sup>**\
> _BooleanOp_ : `&&` | `||`

# String Literals

> **<sup>Syntax</sup>**\
> _StringLiteral_ : `'` _String_ `'`\
> _StringLiteralList_ : `[` [_StringLiteral_] { `,` [_StringLiteral_] } `]`

[_RecordMatcher_]: #record-matcher
[_RecordMatcherSingleton_]: #record-matcher-singleton
[_RecordMatcherGroup_]: #record-matcher-group
[_RecordMatcherNot_]: #record-matcher-not
[_RecordMatcherComposite_]: #record-matcher-composite
[_RecordMatcherTrue_]: #record-matcher-true

[_FieldMatcher_]: #field-matcher
[_FieldMatcherSubfield_]: #field-matcher-subfield
[_FieldMatcherExists_]: #field-matcher-exists

[_SubfieldListMatcher_]: #subfield-list-matcher
[_SubfieldListMatcherSingleton_]: #subfield-list-matcher-singleton
[_SubfieldListMatcherComposite_]: #subfield-list-matcher-composite
[_SubfieldListMatcherGroup_]: #subfield-list-matcher-group
[_SubfieldListMatcherNot_]: #subfield-list-matcher-not

[_TagMatcher_]: #tag-matcher
[_Tag_]: tag.md#tag

[_OccurrenceMatcher_]: #occurrence-matcher
[_OccurrenceDigits_]: occurrence.md#occurrence-digits

[_SubfieldMatcher_]: #subfield-matcher
[_SubfieldCode_]: subfield.md#subfield-code
[_SubfieldCodes_]: subfield.md#subfield-codes

[_ComparisonOp_]: #comparison-operators
[_BooleanOp_]: #boolean-operators

[_StringLiteral_]: #string-literals
[_StringLiteralList_]: #string-literals
