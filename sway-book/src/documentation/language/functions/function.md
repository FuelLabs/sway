# Functions

In this section we will define a function that takes two numerical inputs and returns a [boolean](../built-ins/boolean.md) value indicating whether they are equal. We will also take a look at how to use the function.

## Declaration

The following function is called `equals` and it takes two parameters of type `u64` (64-bit unsigned integers). It performs a comparison and [implicitly](./return.md) returns the result of that comparison.

```sway
{{#include ../../../code/language/functions/src/lib.sw:equals}}
```

## Usage

The following is a way to use the function defined above.

```sway
{{#include ../../../code/language/functions/src/lib.sw:usage}}
```
