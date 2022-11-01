# Enum Memory Layout

Enums have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant.

The space reserved after the tag is equivalent to the size of the _largest_ enum variant.

To calculate the size of an enum in memory, add 8 bytes to the size of the largest variant. For example, in the [`Color`](../../language/built-ins/enums.md) example where the variants are all `()`, the size would be 8 bytes since the size of the largest variant is 0 bytes.
