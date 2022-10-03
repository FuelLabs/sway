# Arrays

An array is similar to a tuple, but an array's values must all be of the same type. It's defined using square brackets `[]` and separating its values using commas.

Arrays are allocated on the stack since their size is known and thus the size is _always_ static, i.e. it cannot change. An array of five elements cannot become an array of six elements.

Unlike a tuple, an array can be iterated over through indexing.

```sway
{{#include ../../../code/language/built-ins/arrays/src/lib.sw:syntax}}
```

> **Note**: Arrays are currently immutable which means that changing elements of an array once initialized is not yet possible.

> TODO: add array mutability to known issues and remove the note above
