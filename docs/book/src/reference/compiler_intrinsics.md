# Compiler Intrinsics

The Sway compiler supports a list of intrinsics that perform various low level operations that are useful for building libraries. Compiler intrinsics should rarely be used but are preferred over `asm` blocks because they are type-checked and safer overall.

Below is a list of all available compiler intrinsics, grouped by category:

## Reflection

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
__assert_is_str_array<T>()
```

**Description:** Throws a compile error if type `T` is not a string array.

**Constraints:** None.

---

```sway
__to_str_array(s: str) -> str[N]
```

**Description:** Converts a string slice `s` to a string array at compile time.

**Constraints:** Parameter `s` must be a string literal.

---

```sway
__size_of<T>() -> u64
```

**Description:** Returns the size of type `T` in bytes.

**Constraints:** None.

---

```sway
__size_of_val<T>(val: T) -> u64
```

**Description:** Returns the size of type `T` in bytes.

**Constraints:** None.

---

```sway
__size_of_str_array<T>() -> u64
```

**Description:** Returns the length `N` of the string array if `T` is a string array `str[N]`, or zero if `T` is not a string array.

## Binary operations

---

```sway
__eq<T>(lhs: T, rhs: T) -> bool
```

**Description:** Returns whether `lhs` and `rhs` are equal.

**Constraints:** `T` is `bool`, `u8`, `u16`, `u32`, `u64`, `u256`, `b256`, or `raw_ptr`.

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
__add<T>(lhs: T, rhs: T) -> T
```

**Description:** Adds `lhs` and `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__sub<T>(lhs: T, rhs: T) -> T
```

**Description:** Subtracts `rhs` from `lhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__mul<T>(lhs: T, rhs: T) -> T
```

**Description:** Multiplies `lhs` by `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__div<T>(lhs: T, rhs: T) -> T
```

**Description:** Divides `lhs` by `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__mod<T>(lhs: T, rhs: T) -> T
```

**Description:** Modulo of `lhs` by `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`.

---

```sway
__and<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise AND `lhs` and `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

---

```sway
__or<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise OR `lhs` and `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

---

```sway
__xor<T>(lhs: T, rhs: T) -> T
```

**Description:** Bitwise XOR `lhs` and `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

---

```sway
__rsh<T>(lhs: T, rhs: u64) -> T
```

**Description:** Logical right shift of `lhs` by `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

---

```sway
__lsh<T>(lhs: T, rhs: u64) -> T
```

**Description:** Logical left shift of `lhs` by `rhs`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

## Unary operations

---

```sway
__not(val: T) -> T
```

**Description:** Bitwise NOT of `val`.

**Constraints:** `T` is an integer type: `u8`, `u16`, `u32`, `u64`, `u256`, or `b256`.

## Blockchain

---

```sway
__log<T>(val: T) where T: AbiEncode
```

**Description:** Logs value `val`.

**Constraints:** `T` must implement `std::codec::AbiEncode`.

---

```sway
__revert(code: u64) -> !
```

**Description:** Reverts with error code `code`.

**Constraints:** None.

---

```sway
__gtf<T>(index: u64, tx_field_id: u64) -> T
```

**Description:** Returns transaction field with ID `tx_field_id` at index `index`, if applicable. This is a wrapper around FuelVM's [`gtf` instruction](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gtf-get-transaction-fields). The resulting field is cast to `T`.

**Constraints:** None.

---

```sway
__smo<T>(recipient: b256, data: T, coins: u64)
```

**Description:** Sends a message `data` of arbitrary type `T` and `coins` amount of the _base asset_ to address `recipient`.

**Constraints:** None.

## Memory handling

---

```sway
__addr_of<T>(val: T) -> raw_ptr
```

**Description:** Returns the address in memory where `val` is stored.

**Constraints:** None.

---

```sway
__alloc<T>(count: u64) -> raw_ptr
```

**Description:** Allocates `count` contiguous elements of `T` on the heap and returns a pointer to the newly allocated memory.

**Constraints** None.

---

```sway
__ptr_add(ptr: raw_ptr, offset: u64) -> raw_ptr
```

**Description:** Adds `offset` to the raw value of pointer `ptr`.

**Constraints:** None.

---

```sway
__ptr_sub(ptr: raw_ptr, offset: u64) -> raw_ptr
```

**Description:** Subtracts `offset` from the raw value of pointer `ptr`.

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
The mutability of reference is defined by the first parameter's mutability.

Runtime bound checks are not generated, and must be done manually when and where appropriated. Compile-time bound checks are done when possible.

**Constraints:**

- `item` is a reference to an array or a slice;
- when `start` is a literal, it must be smaller than `item` length;
- when `end` is a literal, it must be smaller than or equal to `item` length;
- `end` must be greater than or equal to `start`.

---

```sway
__elem_at<T>(item: &[T; N], index: u64) -> &T
__elem_at<T>(item: &[T], index: u64) -> &T
__elem_at<T>(item: &mut [T; N], index: u64) -> &mut T
__elem_at<T>(item: &mut [T], index: u64) -> &mut T
```

**Description:** Returns a reference to the indexed element.

The mutability of reference is defined by the first parameter's mutability.

Runtime bound checks are not generated, and must be done manually when and where appropriated. Compile-time bound checks are done when possible.

**Constraints:**

- `item` is a reference to an array or a slice;
- when `index` is a literal, it must be smaller than `item` length.

## Storage

---

**Without `dynamic_storage` experimental feature enabled**

```sway
__state_load_word(key: b256) -> u64
```

**Description:** Reads and returns a single word from storage at key `key`. If the storage slot at key `key` is not set, zero is returned.

**Constraints:** None.

**With `dynamic_storage` experimental feature enabled**

```sway
__state_load_word(key: b256, offset: u64) -> u64
```

**Description:** Reads and returns a single word from storage at key `key` and offset `offset`. If the storage slot at key `key` is not set, zero is returned.

**Constraints:** `offset` must be a valid word offset inside of the storage slot boundaries. E.g., if the storage slot contains four words, the lowest offset is zero, and the highest is three.

---

```sway
__state_store_word(key: b256, val: u64) -> bool
```

**Description:** Stores a single word `val` into storage at key `key`. Returns a Boolean describing whether the store slot was previously set.

**Constraints:** None.

---

```sway
__state_load_quad(key: b256, ptr: raw_ptr, slots: u64) -> bool
```

**Description:** Reads `slots` number of slots (32 bytes each) from storage starting at key `key` and stores them in memory starting at address `ptr`. Returns a boolean describing whether all the storage slots were previously set.

**Constraints:** None.

---

```sway
__state_store_quad(key: b256, ptr: raw_ptr, slots: u64) -> bool
```

**Description:** Stores `slots` number of slots (32 bytes each) starting at address `ptr` in memory into storage starting at key `key`. Returns a boolean describing whether the first storage slot was previously set.

**Constraints:** None.

---

```sway
__state_load_slot(key: b256, ptr: raw_ptr, offset: u64, len: u64) -> bool
```

**Description:** Reads `len` bytes from the storage slot at key `key` starting at byte `offset` into memory at address `ptr`. Returns `true` if the slot was previously set, `false` otherwise. If the slot was not previously set, memory is not modified. Maps to the `SRDD` or `SRDI` opcode, depending on whether `len` is a constant that fits in a twelve bit immediate.

**Constraints:** None.

---

```sway
__state_store_slot(key: b256, ptr: raw_ptr, len: u64)
```

**Description:** Writes `len` bytes starting at address `ptr` in memory into the storage slot at key `key`. Maps to the `SWRD` or `SWRI` opcode, depending on whether `len` is a constant that fits in a twelve bit immediate.

**Constraints:** None.

---

```sway
__state_update_slot(key: b256, ptr: raw_ptr, offset: u64, len: u64)
```

**Description:** Updates `len` bytes at byte `offset` in the storage slot at key `key` with data from memory at address `ptr`. If `offset` is `u64::max()`, appends the data after the existing content. Maps to the `SUPD` or `SUPI` opcode, depending on whether `len` is a constant that fits in a twelve bit immediate.

**Constraints:** None.

---

```sway
__state_clear(key: b256, slots: u64) -> bool
```

**Description:** Clears `slots` number of non-dynamic storage slots (32 bytes each) starting at key `key`. Returns a boolean describing whether all the storage slots were previously set. Maps to the `SCWQ` opcode.

If the return value is not needed, use `__state_clear_slots` instead, for it is less gas consuming.

**Constraints:** None.

---

```sway
__state_clear_slots(key: b256, slots: u64)
```

**Description:** Clears `slots` number of dynamic storage slots starting at key `key`. Unlike `__state_clear`, it does not report whether slots were previously set, making it less gas consuming. Maps to the `SCLR` opcode.

**Constraints:** None.

---

```sway
__state_preload(key: b256) -> u64
```

**Description:** Preloads the storage slot at key `key` and returns the length of the stored data. If the storage slot at key `key` is not set, zero is returned. Maps to the `SPLD` opcode.

**Constraints:** None.

## Miscellaneous

---

```sway
__jmp_mem()
```

**Description:** Jumps to `MEM[$hp]`.

**Constraints:** None.

---

```sway
__dbg<T>(value: T) -> T where T: Debug
```

**Description:** Automatically calls the `Debug`'s trait `fmt` method on the passed `value`, with file, line and column information. The passed value is returned without any modification, allowing `__dbg(...)` to be used inside of any expression.

The code generated by this intrinsic function varies with the compilation mode. For example:

```terminal
forc build            <- will print everything as expected
forc build --release  <- nothing will be printed
```

To enable code generation even on `--release` builds, the flag `force-dbg-in-release` needs to be enabled inside `Forc.toml`, in the `[project]` section:

```toml
[project]
force-dbg-in-release = true
```

It is strongly recommended to always remove this flag before publishing binaries as it will not have any effect when running on real nodes and it only increases gas usage.

**Constraints:** `T` must implement `std::debug::Debug`.

---

```sway
__transmute<A, B>(src: A) -> B
```

**Description:** Reinterprets the bits of the value `src` of type `A` as another type `B`.

**Constraints:** `A` and `B` must have the exactly same size.

---

```sway
__runtime_mem_id<T>() -> u64
__encoding_mem_id<T>() -> u64
```

**Description:** Returns an opaque number that identifies the memory representation of a type. No information is conveyed by this number and it should only be compared for equality.

This number is not guaranteed to be stable on different compiler versions.

`__runtime_mem_id` represents how the type is represented inside the VM's memory.

`__encoding_mem_id` represents how the type is encoded. It returns zero when type does not have encoding representation.

**Constraints:** None.
