# Pattern Matching

The following examples present pattern matching using the [`match`](../../language/control-flow/match/index.md) keyword for the catch-all case.

## Encouraged

The `_` is used for the catch-all to indicate the important cases have been defined above and the last case is not important enough to warrant a name.

```sway
{{#include ../../../code/language/style-guide/pattern_matching/src/lib.sw:style_match_unnamed}}
```

## Alternative

We may apply an appropriate name to provide context to the reader; however, unless it provides additional information the preferred usage is defined in the [`encouraged`](#encouraged) case.

```sway
{{#include ../../../code/language/style-guide/pattern_matching/src/lib.sw:style_match_named}}
```
