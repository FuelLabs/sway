# Arrays

An array is similar to a tuple, but an array's values must all be of the same type. It's defined using square brackets `[]` and separating its values using commas.

Unlike a tuple, an array can be iterated over through indexing.

```sway
{{#include ../../../code/language/built-ins/arrays/src/lib.sw:syntax}}
```

Arrays are allocated on the stack and thus the size of an array is considered to be `static`. What this means is that once an array is declared to have a size of `n` it cannot grow to contain more, or fewer, elements than `n`. The size of the array cannot change.
