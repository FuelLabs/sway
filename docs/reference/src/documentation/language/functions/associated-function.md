# Associated Functions

Associated functions are similar to methods in that they are also defined in the context of a [struct](../built-ins/structs.md) or [enum](../built-ins/enums.md), but they do not use any of the data in the struct and as a result do not take `self` as a parameter.

Associated functions could be standalone functions, but they are included in a specific type for organizational or semantic reasons.

## Declaration

In this example we will take a look at a struct; however, an enum will work in the same way.

```sway
{{#include ../../../code/language/functions/src/lib.sw:struct_definition}}
```

We start by using the `impl` (implementation) keyword, followed by the name of our struct, to define a function that belongs to our object i.e. a method.

```sway
{{#include ../../../code/language/functions/src/lib.sw:associated_impl}}
```

## Usage

To call an associated function on a type we use the following syntax.

```sway
{{#include ../../../code/language/functions/src/lib.sw:associated_usage}}
```
