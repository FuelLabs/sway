# Enum Memory Layout

Enums have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant.

The space reserved after the tag is equivalent to the size of the _largest_ enum variant. To calculate the size of an enum in memory, add 8 bytes to the size of the largest variant.

## Examples

The following examples consist of [enums](../../language/built-ins/enums.md) with two variants.

The largest variant for [`Example One`](#example-one) is the [`u64`](../../language/built-ins/numeric.md) and [`b256`](../../language/built-ins/b256.md) for [`Example Two`](#example-two).

### Example One

The size of enum `T` is `16 bytes`, `8 bytes` for the tag and `8 bytes` for the [`u64`](../../language/built-ins/numeric.md).

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:u64_example}}
```

Instantiating the [`u64`](../../language/built-ins/numeric.md) type will take up `16 bytes`.

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:u64_type_space}}
```

Instantiating the `unit` type will take up `16 bytes`.

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:u64_unit_space}}
```

### Example Two

The size of enum `K` is `40 bytes`, `8 bytes` for the tag and `32 bytes` for the [`b256`](../../language/built-ins/b256.md).

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:b256_example}}
```

Instantiating the [`b256`](../../language/built-ins/b256.md) type will take up `40 bytes`.

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:b256_type_space}}
```

Instantiating the `unit` type will take up `40 bytes`.

```sway
{{#include ../../../code/misc/advanced-concepts/enums/src/lib.sw:b256_unit_space}}
```
