# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize()` and `increment()` functions return the currently set value.

```bash
forc init --template counter my_counter_project
```

The use of `storage` here is **new and is still being stabilized**, please see the [Subcurrency](./subcurrency.md) example for writing storage manually.

```sway
{{#include ../../../examples/counter/src/main.sw}}
```
