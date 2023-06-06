# Naming Convention

A naming convention is a set of rules used to standardize how code is written.

## CapitalCase

[Structs](../built-ins/structs.md), [traits](../traits/index.md), and [enums](../built-ins/enums.md) are `CapitalCase` which means each word has a capitalized first letter. The fields inside a struct should be [snake_case](#snake_case) and `CapitalCase` inside an enum.

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:structures}}
```

## snake_case

Modules, [variables](../variables/index.md), and [functions](../functions/index.md) are `snake_case` which means that each word is lowercase and separated by an underscore.

Module name:

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:module}}
```

Function and variable:

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:function_case}}
```

## SCREAMING_SNAKE_CASE

[Constants](../variables/const.md) are `SCREAMING_SNAKE_CASE` which means that each word in capitalized and separated by an underscore.

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:const}}
```
