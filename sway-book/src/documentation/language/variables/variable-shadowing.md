# Shadowing

When looking at the [let](let.md) variable we've seen that the value can be changed through the use of the `mut` keyword. We can take this a couple steps further through [reassignment](#reassignment) and [variable shadowing](#variable-shadowing).

## Reassignment

We can redefine the type and value of a variable by instantiating a new version after the first declaration.

```sway
{{#include ../../../code/language/variables/src/lib.sw:reassignment}}
```

## Variable Shadowing

If we do not want to alter the original variable but we'd like to temporarily reuse the variable name then we can use block scope to constrain the variable.

```sway
{{#include ../../../code/language/variables/src/lib.sw:shadowing}}
```

`foo` & `bar` defined inside the curly brackets `{}` only exist inside the `{}` scope therefore the original `foo` & `bar` with the value of `5` maintain their value.
