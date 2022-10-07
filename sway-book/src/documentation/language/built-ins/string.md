# String Type

A string is a collection of characters (letters, numbers etc.).

Sway has one string type and it's a fixed length string which has the following implications:

- A string must have a hardcoded length
- A string cannot be grown or shrunk during execution
- The content of the string must meet its length
  - This could be via a legitimate value that takes up the entire length or through padding

The reason for this is that the compiler must know the size of the type and the length is a part of the type.

## Examples

The following three variables show how a string can be instantiated. The length of each string is placed inside the `[]` to let the compiler know that the type is a string of length `[<length>]`.

```sway
{{#include ../../../code/language/built-ins/strings/src/lib.sw:explicit}}
```

It can be seen that the variable `fuel` is a string of length four because "fuel" has four characters (f, e, u and l).

The example above is a demonstration which emphasizes the fixed-length aspect of strings however the compiler is smart enough to infer the size and thus the string type (and length) does not need to be specified.

```sway
{{#include ../../../code/language/built-ins/strings/src/lib.sw:implicit}}
```

Strings default to UTF-8 in Sway.
