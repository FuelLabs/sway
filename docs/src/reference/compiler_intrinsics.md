# Compiler Intrinsics

The Sway compiler supports a list of intrinsics that perform various low level operations that are useful for building libraries. Compiler intrinsics should rarely be used but are preferred over `asm` blocks because they are type-checked and are safer overall. Below is a list of all available compiler intrinsics:

___

```sway
__size_of_val<T>(val: T) -> u64
```

**Description:** Return the size of type `T` in bytes.

**Constraints:** None.

___

```sway
__size_of<T>() -> u64`
```

**Description:** Return the size of type `T` in bytes.

**Constraints:** None.

___

```sway
__is_reference_type<T>() -> bool
```

**Description:** Returns `true` if `T` is a _reference type_ and `false` otherwise.

**Constraints:** None.

___


```sway
__eq<T>(lhs: T, rhs: T) -> bool`
```

**Description:** Returns whether `lhs` and `rhs` are equal.

**Constraints:** `T` is `bool`, `u8`, `u16`, `u32`, `u64`, or `raw_ptr`.

___

```sway
__gtf<T>(index: u64, tx_field_id: u64) -> T`
```

**Description:** Returns transaction field with ID `tx_field_id` at index `index`, if applicable. This is a wrapper around FuelVM's [`gtf` instruction](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gtf-get-transaction-fields). The resuting field is cast to `T`.

**Constraints:** None.

___

```sway
__addr_of<T>(val: T) -> raw_ptr`
```

**Description:** Returns the address in memory where `val` is stored.

**Constraints:** `T` is a reference type.

___

```sway
__state_load_word(key: b256) -> u64`
```

**Description:** Reads and returns a single word from storage at key `key`.

**Constraints:** None.

___

```sway
__state_load_quad(key: b256, ptr: raw_ptr)`
```

**Description:** Reads a `b256` from storage at key `key` and stores it in memory at address `raw_ptr` 

**Constraints:** None.

___

```sway
__state_store_word(key: b256, val: u64)`
```

**Description:** Stores a single word `val` into storage at key `key`.

**Constraints:** None.

___

```sway
__state_store_quad(key: b256, ptr: raw_ptr)`
```

**Description:** Stores a `b256` from address `ptr` in memory into storage at key `key`.

**Constraints:** None.

___

```sway
__log<T>(val: T)`
```

**Description:** Logs value `val`.

**Constraints:** None.

___

```sway
__add<T>(lhs: T, rhs: T) -> T`
```

**Description:** Adds `lhs` and `rhs` and returns the result.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.

___

```sway
__sub<T>(lhs: T, rhs: T) -> T`
```

**Description:** Subtracts `rhs` from `lhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.

___

```sway
__mul<T>(lhs: T, rhs: T) -> T`
```

**Description:** Multiplies `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.

___

```sway
__div<T>(lhs: T, rhs: T) -> T`
```

**Description:** Divides `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.

___

```sway
__revert(code: u64)`
```

**Description:** Reverts with error code `code`.

**Constraints:** None.

___

```sway
__ptr_add(ptr: raw_ptr, offset: u64)`
```

**Description:** Adds `offset` to the raw value of pointer `ptr`. 

**Constraints:** None.

___

```sway
__ptr_sub(ptr: raw_ptr, offset: u64)`
```

**Description:** Subtracts `offset` to the raw value of pointer `ptr`. 

**Constraints:** None.

___
