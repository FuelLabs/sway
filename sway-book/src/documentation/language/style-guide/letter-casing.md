# Letter Casing

> TODO: intro

## CapitalCase

Structs, traits, and enums are `CapitalCase` which means each word has a capitalized first letter. The fields inside a struct should be [snake_case](#snake_case) and `CapitalCase` inside an enum.

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:structures}}
```

## snake_case

Modules, variables, and functions are `snake_case` which means that each word is lowercase and separated by an underscore.

Module name:

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:module}}
```

Function and variable:

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:function_case}}
```

## SCREAMING_SNAKE_CASE

Constants are `SCREAMING_SNAKE_CASE` which means that each word in capitalized and separated by an underscore.

```sway
{{#include ../../../code/language/style-guide/letter_casing/src/lib.sw:const}}
```
