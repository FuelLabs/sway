# Returning from functions

In the previous sections we have seen how functions return values without much information. In this section we will take a closer look at how we can return data from a function.

There are two ways to return:

<!-- no toc-->
- [Explicitly](#explicit-return)
- [Implicitly](#implicit-return)

When returning data from a function the return types must match up with the return types declared in the function signature. This means that if the first return type is a `u64` then the type of the first value being returned must also be a `u64`.

## Explicit Return

To return from a function explicitly we use the `return` keyword followed by the arguments and a semi-colon.

```sway
{{#include ../../../code/language/functions/src/explicit.sw:main}}
```

A return expression is typically used at the end of a function however as long as the syntax is correct it can be used anywhere inside a function. This can be used as a mechanism to exit the function early if some condition is met.

```sway
{{#include ../../../code/language/functions/src/explicit.sw:return_data}}
```

## Implicit Return

To return from a function implicitly we do not use the `return` keyword and we omit the ending semi-colon at the end of the line.

```sway
{{#include ../../../code/language/functions/src/explicit.sw:main}}
```

Similarly to the explicit usage of a `return` this will typically be used at the end of a function but it can also be used anywhere. 

```sway
{{#include ../../../code/language/functions/src/explicit.sw:return_data}}
```
