# Behavior Considered Undefined

Sway code that contains any of the following behavior is considered undefined.
The compiler is allowed to treat undefined Sway code however it desires,
including removing it or replacing it with any other Sway code.

This is not an exhaustive list, it may grow or shrink, there is no formal model
of Sway's semantics so there may be more behavior considered undefined. We
reserve the right to make some of the listed behavior defined in the future.

* Invalid arithmetic operations (overflows, underflows, division by zero, etc.).
* Misuse of compiler intrinsics.
* Incorrect use of inline assembly.
* Reading and writing `raw_ptr` and `raw_slice`.
* Slicing and indexing out of bounds by directly using compiler intrinsics.
* Modifying collections while iterating over them using `Iterator`s.
