# Store & Get

Storage can be manipulated directly through the use of `store()` & `get()` functions. They utilize [generics](../../../language/generics/index.md) to store and retrieve values.

## Declaration

To use `store()` & `get()` we must import them however we are not required to declare a `storage` block.

```sway
{{#include ../../../../code/operations/storage/store_get/src/main.sw:import}}
```

## Store

To store a generic value `T` we must provide a key of type `b256`. 

In this example we store some number of type `u64`.

```sway
{{#include ../../../../code/operations/storage/store_get/src/main.sw:store}}
```

## Get

To retrieve a generic value `T` at the position of `key` we must specify the type that we are retrieving.

In this example we retrieve some `u64` at the position of `key`.

```sway
{{#include ../../../../code/operations/storage/store_get/src/main.sw:get}}
```
