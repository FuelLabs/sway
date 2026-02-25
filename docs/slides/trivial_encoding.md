---
layout: cover
marp: true
---

<!-- markdownlint-disable -->
# Trivially Encodable/Decodable Types

- Encoding Recap
- Trivially Encodable/Decodable Types

---

# Encoding Recap

When you call a contract like 

```rust
let caller = abi(OtherContract, external_contract_id.into());
let result = caller.external_call(1);
```

the compiler desugar this into:

```rust
let args_slice: raw_slice = encode(1);
let result_slice: raw_slice = caller.external_call(args_slice);
let result = u64::abi_decode(result_slice);
```

see more at: https://github.com/FuelLabs/sway/blob/master/docs/slides/encoding.md

---

This means that encoding/decoding are *NOT* free.
- they increase binary size, and;
- they increase gas usage.

Unless...

- arguments are trivially encodable, and;
- return type is trivially decodable.

---

# Trivially Encodable/Decodable Types

A type is trivially encodable/decodable if its runtime memory representation is exactly the same and its encoded representation (with one caveat explored later).

It is the same idea as `zero copy deserialization`.

> Zero-copy deserialization is a technique that allows data to be accessed directly from a serialized byte buffer without allocating new memory or copying data into a separate structure. This is achieved by ensuring the serialized format's memory layout matches the in-memory representation of the target data structure, enabling direct casting or pointer offsets to access fields without any parsing or transformation work.

---

# Trivially Encodable/Decodable Types

Example:

```rust
fn main ( _: (1u64, 2u64, 3u64) ) { ... }
```

Runtime Representation

```
-------------------------------------
| 00 ... 01 | 00 ... 02 | 00 ... 03 |
-------------------------------------
```

Encoded Representation

```
-------------------------------------
| 00 ... 01 | 00 ... 02 | 00 ... 03 |
-------------------------------------
```

---

# Trivially Encodable/Decodable Types


```rust
fn main ( _: (1u8, 2u64, 3u64) ) { ... }
```

Runtime Representation

```
------------------------------------------
| 01 | 00 ... 00 | 00 ... 02 | 00 ... 03 |
------------------------------------------
       ^^^^^^^^^ padding
```

Encoded Representation

```
------------------------------
| 01 | 00 ... 02 | 00 ... 03 |
------------------------------
```

---

Because the second example has a mismatch, we can easily see the cost of encoding on the binary size.

```
Finished release [optimized + fuel] target(s) [136 B] in 0.90s
```

versus

```
Finished release [optimized + fuel] target(s) [208 B] in 0.89s
```

---

How this works under the hood?

```rust
pub trait AbiEncode {
    fn is_encode_trivial() -> bool;
    fn abi_encode(self, buffer: Buffer) -> Buffer;
}
pub trait AbiDecode {
    fn is_decode_trivial() -> bool;
    fn abi_decode(ref mut buffer: BufferReader) -> Self;
}
pub fn encode<T>(item: T) -> raw_slice where T: AbiEncode {
    if T::is_encode_trivial() { ... } else { ... }
}
pub fn abi_decode<T>(data: raw_slice) -> T where T: AbiDecode {
    if T::is_decode_trivial() { ... } else { ... }
}
```

---

# Trap Representations

> A *trap representation* is a bit pattern that is *not* a valid value for the type. 

Some types *DO* match their memory layout, but they stil *CANNOT* be safe trivially decoded.
Example: `bool`. It is trivially encodable, but not trivially decodable.
Another example are `enums`, because they have a "hidden" discriminant that only accepts certain values.

```rust
enum A { A: ..., B: ..., C: ... }
```
```
--------------------------
| 0000000000000000 | ... |
--------------------------
  ^^^^^^^^^^^^^^^^ Discriminant (8 bytes)
```

---

If a developer is OK with the risks and wants to deal with invalid representation manually, he can force a type by doing:

```rust
pub struct TriviallyDecodable<T> { value: T }
impl<T> AbiDecode for TriviallyDecodable<T> {
      fn is_decode_trivial() -> bool { true }
      fn abi_decode(ref mut buffer: BufferReader) -> Self {
            let value = T::abi_decode(buffer);
            Self { value }
      }
}

fn main(_: TriviallyDecodable<bool>) { ... }
```
