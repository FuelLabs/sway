# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize_counter()` and `increment_counter()` ABI methods return the currently set value.

```bash
forc template --template-name counter my_counter_project
```

```sway
{{#include ../../../examples/counter/src/main.sw}}
```
