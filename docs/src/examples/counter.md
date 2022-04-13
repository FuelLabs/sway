# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize()` and `increment()` functions return the currently set value.

The use of `storage` here is **new and is still being stabalized**, please see the [Subcurrency](./subcurrency.md) example for writing storage by-hand.

```sway
{{#include ../../../examples/counter/src/main.sw}}
```
