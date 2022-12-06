# Counter

The following example implements a counter which is able to:

- Increment the count by 1
- Decrement the count by 1
- Retrieve the value of the counter

## ABI

To create a counter we must define an [`ABI`](../language/program-types/contract.md) which exposes methods that manipulate the count and retrieve its value. Since we are handling [`storage`](../operations/storage/index.md) we must provide [`storage annotations`](../language/annotations/attributes/storage.md) on the functions.

```sway
{{#include ../../code/examples/counter/src/main.sw:abi}}
```

## Implementation

We initialize a count in [`storage`](../operations/storage/index.md) with the value of zero and implement methods to increment & decrement the count by one and return the value.

```sway
{{#include ../../code/examples/counter/src/main.sw:counter}}
```
