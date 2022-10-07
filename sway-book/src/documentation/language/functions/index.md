# Functions, methods, and associated functions

Functions, and by extension methods and associated functions, are a way to group functionality together in a way that allows for code reuse without having to re-write the code in each place that it is used.

The distinction between a function, method and associated function is as follows:

- A [function](function.md) is a grouping of code that is independent of any object
- A [method](method.md) is a function that is associated with an object and it uses `self` as the first parameter
- An [associated function](associated-function.md) is a method but without the `self` parameter

## Function Decleration

A function decleration consists of a few components

- The `fn` keyword
- A unqiue name for the function
- Comma separated optional parameters, and their types, inside `()`
- An optional return type

Here is a template that applies to the aforementioned functions.

```sway
{{#include ../../../code/language/functions/src/lib.sw:definition}}
```
