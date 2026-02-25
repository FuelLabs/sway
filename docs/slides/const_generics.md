---
layout: cover
marp: true
---

<!-- markdownlint-disable -->
# Const Generics

`const generics` let you parameterize types and functions with compile‑time constant values, enabling generic code over sizes, indices, or other fixed values.

```rust
#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> AbiEncode for [T; N] where T: AbiEncode {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < N {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
```

---

This allowed us to implement all the `std` traits for arrays, string arrays etc...

- AbiEncode
- AbiDecode
- Clone
- Debug
- Hash
- Iterator
- PartialEq
- Eq
- etc...

---

"const generics" can be used on structs, enums, functions, impls etc...

```rust
struct S<T, const N: u64> {}

impl<const Z: u64> OneVariant<Z> {
    pub fn return_n(self) -> u64 {
        Z
    }
}

enum OneVariant<const N: u64> {
    A: [u64; N],
}

fn const_with_const_generics<const B: u64>() {
    const A: u64 = B + 1;
    let _ = __dbg(A);
}
```

---

and can be used like any other constant.

```rust
impl<T, const N: u64> AbiDecode for [T; N] {
    ...
    fn abi_decode(ref mut buffer: BufferReader) -> [T; N] {
        const LENGTH: u64 = __size_of::<T>() * N;
        let mut array = [0u8; LENGTH];
        ...
    }
}
```