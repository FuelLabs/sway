# Types

Sway is a compiled language and as such each data structure has a definition i.e. a `type` which has some `size` that must be allocated on the stack.

The compiler can usually infer the `type` based on its usage however there may be occasions where the compiler cannot make the inference or the developer may deem it more useful to explicitly annotate a variable in order to make the code easier to read.

Annotating a variable is done by placing the annotation after the variable name but before the assignment (the `=` sign).

```sway
{{#include ../../../code/language/annotations/src/main.sw:example}}
```

The compiler will disallow incorrect `type` annotations therefore replacing the `bool` annotation on the variable `baz` with a `u64` will result in a compilation error.
