# Arrays

An array is similar to a tuple, but an array's values must all be of the same type. Arrays can hold arbitrary types include non-primitive types.

An array is written as a comma-separated list inside square brackets:

```sway
let x = [1, 2, 3, 4, 5];
```

Arrays are allocated on the stack since their size is known. An array's size is _always_ static, i.e. it cannot change. An array of five elements cannot become an array of six elements.

Arrays can be iterated over, unlike tuples. An array's type is written as the type the array contains followed by the number of elements, semicolon-separated and within square brackets, e.g. `[u64; 5]`. To access an element in an array, use the _array indexing syntax_, i.e. square brackets.

```sway
{{#include ../../../examples/arrays/src/main.sw}}
```

> **Note**: Arrays are currently immutable which means that changing elements of an array once initialized is not yet possible.
