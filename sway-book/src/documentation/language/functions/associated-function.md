# Associated Functions

Associated functions are similar to methods in that they are also defined in the context of a [struct](../built-ins/structs.md) or [enum](../built-ins/enums.md), but they do not use any of the data in the struct and as a result do not take `self` as a parameter. 

Associated functions could be standalone functions, but they are included in a specific type for organizational or semantic reasons.

### Decleration

In this example we will take a look at a struct however an enum will work in the same way.

```sway
{{#include ../../../code/language/functions/src/lib.sw:struct_definition}}
```

We start by using the `impl` (implementation) keyword, followed by the name of our struct, to define a function that belongs to our object i.e. a method.

```sway
{{#include ../../../code/language/functions/src/lib.sw:associated_impl}}
```

### Usage

The syntax to call an associated function is unique to Sway. 

When referring to a type directly you preface the type with a tilde `~` and use two colons `::` after the type followed by the name of the associated function.

```sway
{{#include ../../../code/language/functions/src/lib.sw:associated_usage}}
```
