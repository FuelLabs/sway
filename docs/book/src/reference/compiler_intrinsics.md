# Compiler Intrinsics

The Sway compiler supports a list of intrinsics that perform various low level operations that are useful for building libraries. Compiler intrinsics should rarely be used but are preferred over `asm` blocks because they are type-checked and are safer overall. Below is a list of all available compiler intrinsics:

---

```sway
__size_of_val<T>(val: T) -> u64
```

**Description:** Return the size of type `T` in bytes.

**Constraints:** None.

---

```sway
__size_of<T>() -> u64
```

**Description:** Return the size of type `T` in bytes.

**Constraints:** None.

---

```sway
__size_of_str_array<T>() -> u64
```

**Description:** Return the size of type `T` in bytes. This intrinsic differs from `__size_of` in the case of "string arrays" where the actual length in bytes of the string is returned without padding the byte size to the next word alignment. When `T` is not a "string array" `0` is returned.

**Constraints:** None.

---

```sway
__assert_is_str_array<T>()
```

**Description:** Throws a compile error if type `T` is not a "string array".

**Constraints:** None.

---

```sway
__to_str_array(s: str) -> str[N]
```

**Description:** Converts a "string slice" to "string array" at compile time. Parameter "s" must be a string literal.

**Constraints:** None.

---

```sway
__is_reference_type<T>() -> bool
```

**Description:** Returns `true` if `T` is a _reference type_ and `false` otherwise.

**Constraints:** None.

---

```sway
__is_str_array<T>() -> bool
```

**Description:** Returns `true` if `T` is a string array and `false` otherwise.

**Constraints:** None.

---

```sway
__eq<T>(lhs: T, rhs: T) -> bool
```

**Description:** Returns whether `lhs` and `rhs` are equal.

**Constraints:** `T` is `bool`, `u8`, `u16`, `u32`, `u64`, `u256`, `b256` or `raw_ptr`.

---

```sway
__gt<T>(lhs: T, rhs: T) -> bool
```

**Description:** Returns whether `lhs` is greater than `rhs`.

**Constraints:** `T` is `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__lt<T>(lhs: T, rhs: T) -> bool
```

**Description:** Returns whether `lhs` is less than `rhs`.

**Constraints:** `T` is `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__gtf<T>(index: u64, tx_field_id: u64) -> T
```

**Description:** Returns transaction field with ID `tx_field_id` at index `index`, if applicable. This is a wrapper around FuelVM's [`gtf` instruction](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gtf-get-transaction-fields). The resulting field is cast to `T`.

**Constraints:** None.

---

```sway
__addr_of<T>(val: T) -> raw_ptr
```

**Description:** Returns the address in memory where `val` is stored.

**Constraints:** None.

---

```sway
__state_load_word(key: b256) -> u64
```

**Description:** Reads and returns a single word from storage at key `key`.

**Constraints:** None.

---

```sway
__state_load_quad(key: b256, ptr: raw_ptr, slots: u64) -> bool
```

**Description:** Reads `slots` number of slots (`b256` each) from storage starting at key `key` and stores them in memory starting at address `ptr`. Returns a Boolean describing whether all the storage slots were previously set.

**Constraints:** None.

---

```sway
__state_store_word(key: b256, val: u64) -> bool
```

**Description:** Stores a single word `val` into storage at key `key`. Returns a Boolean describing whether the store slot was previously set.

**Constraints:** None.

---

```sway
__state_store_quad(key: b256, ptr: raw_ptr, slots: u64) -> bool
```

**Description:** Stores `slots` number of slots (`b256` each) starting at address `ptr` in memory into storage starting at key `key`. Returns a Boolean describing whether the first storage slot was previously set.

**Constraints:** None.

---

```sway
__log<T>(val: T) where T: AbiEncode
```

**Description:** Logs value `val`.

**Constraints:**

- `T` must implement AbiEncode

---

```sway
__add<T>(lhs: T, rhs: T) -> T
```

**Description:** Adds `lhs` and `rhs` and returns the result.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__sub<T>(lhs: T, rhs: T) -> T
```

**Description:** Subtracts `rhs` from `lhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__mul<T>(lhs: T, rhs: T) -> T
```

**Description:** Multiplies `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__div<T>(lhs: T, rhs: T) -> T
```

**Description:** Divides `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__and<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise AND `lhs` and `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__or<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise OR `lhs` and `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__xor<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise XOR `lhs` and `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__mod<T>(lhs: T, rhs: T) -> T
```

**Description:** Modulo of `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__rsh<T>(lhs: T, rhs: u64) -> T
```

**Description:** Logical right shift of `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__lsh<T>(lhs: T, rhs: u64) -> T
```

**Description:** Logical left shift of `lhs` by `rhs`.

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__revert(code: u64)
```

**Description:** Reverts with error code `code`.

**Constraints:** None.

---

```sway
__ptr_add(ptr: raw_ptr, offset: u64)
```

**Description:** Adds `offset` to the raw value of pointer `ptr`.

**Constraints:** None.

---

```sway
__ptr_sub(ptr: raw_ptr, offset: u64)
```

**Description:** Subtracts `offset` to the raw value of pointer `ptr`.

**Constraints:** None.

---

```sway
__smo<T>(recipient: b256, data: T, coins: u64)
```

**Description:** Sends a message `data` of arbitrary type `T` and `coins` amount of the base asset to address `recipient`.

**Constraints:** None.

---

```sway
__not(op: T) -> T
```

**Description:** Bitwise NOT of `op`

**Constraints:** `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`, `u256`, `b256`.

---

```sway
__jmp_mem()
```

**Description:** Jumps to `MEM[$hp]`.

**Constraints:** None.

---

```sway
__slice<T>(item: &[T; N], start: u64, end: u64) -> &[T]
__slice<T>(item: &[T], start: u64, end: u64) -> &[T]
__slice<T>(item: &mut [T; N], start: u64, end: u64) -> &mut [T]
__slice<T>(item: &mut [T], start: u64, end: u64) -> &mut [T]
```

**Description:** Slices an array or another slice.

This intrinsic returns a reference to a slice containing the range of elements inside `item`.
The mutability of reference is defined by the first parameter mutability.

Runtime bound checks are not generated, and must be done manually when and where appropriated. Compile time bound checks are done when possible.

**Constraints:**

- `item` is an array or a slice;
- when `start` is a literal, it must be smaller than `item` length;
- when `end` is a literal, it must be smaller than or equal to `item` length;
- `end` must be greater than or equal to `start`

---

```sway
__elem_at<T>(item: &[T; N], index: u64) -> &T
__elem_at<T>(item: &[T], index: u64) -> &T
__elem_at<T>(item: &mut [T; N], index: u64) -> &mut T
__elem_at<T>(item: &mut [T], index: u64) -> &mut T
```

**Description:** Returns a reference to the indexed element. The mutability of reference is defined by the first parameter mutability.

Runtime bound checks are not generated, and must be done manually when and where appropriated. Compile time bound checks are done when possible.

**Constraints:**

- `item` is a reference to an array or a reference to a slice;
- when `index` is a literal, it must be smaller than `item` length;

---

```sway
__dbg<T>(value: T) -> T where T: Debug
```

**Description:** Automatically calls the `Debug` trait on the passed `value`, with file, line and column information. The passed value is returned without any modification, allowing `__dbg(...)` to be used inside of any expression.

The code generated by this intrinsic function varies with the compilation mode. For example:

```terminal
forc build            <- will print everything as expected
forc build --release  <- nothing will be printed
```

To enable code generation even on `Release` builds, the flag `force-dbg-in-release` needs to be enabled inside `forc.toml`.
Example:

```toml
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
license = "Apache-2.0"
entry = "main.sw"
name = "some-project"
force-dbg-in-release = true
```

It is strongly suggested to always remove this flag before publishing binaries as it will not have any effect when running
on real nodes and it only increases gas usage.

**Constraints:**

- `T` must implement Debug
