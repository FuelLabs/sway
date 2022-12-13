# String Type

A string is a collection of characters (letters, numbers etc.).

Sway has one string type and it's a fixed length string which has the following implications:

- A string cannot be grown or shrunk during execution
- The content of the string must meet its length
  - This could be via a legitimate value that takes up the entire length or through padding

The reason for this is that the compiler must know the size of the type and the length is a part of the type.

A string can be created through the use of double-quotation marks `"` around the text. The length of the string is permanently set at that point and cannot be changed even if the variable is marked as mutable.

```sway
{{#include ../../../code/language/built-ins/strings/src/lib.sw:implicit}}
```

Strings default to UTF-8 in Sway.
