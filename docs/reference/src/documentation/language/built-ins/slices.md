# Slices

A slice is similar to an [array](arrays.md), in the sense that it is a contiguous sequence of elements of the same type.

Unlike arrays, slices cannot be allocated, because its size is unknown at compilation time. The only way to use slices is through references to a slice.

References to slice are "fat pointers" containing two items:

- a pointer to the first element of the slice;
- a `u64` with how many elements the slice has.

```sway
{{#include ../../../code/language/built-ins/slices/src/lib.sw:syntax}}
```
