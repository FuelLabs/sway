# Methods

Methods are defined within the context of a [struct](../built-ins/structs.md) (or [enum](../built-ins/enums.md)) and either refer to the type or mutate it.

The first parameter of a method is always `self`, which represents the instance of the type the method is being called on.

## Decleration

In this example we will take a look at a struct however an enum will work in the same way.

```sway
{{#include ../../../code/language/functions/src/lib.sw:struct_definition}}
```

We start by using the `impl` (implementation) keyword, followed by the name of our struct, to define a function that belongs to our object i.e. a method.

```sway
{{#include ../../../code/language/functions/src/lib.sw:method_impl}}
```

## Usage

To call a method use the dot syntax: `<variable name>.<method name>()`.

```sway
{{#include ../../../code/language/functions/src/lib.sw:method_usage}}
```
