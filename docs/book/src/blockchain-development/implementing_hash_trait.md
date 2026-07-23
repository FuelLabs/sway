# Implementing the `Hash` Trait

The `Hash` trait from the standard library (`std::hash::Hash`) defines how a value is hashed. Implementing it for your own types lets you hash them with the `sha256` and `keccak256` functions and use them wherever a deterministic hash is required, for example as keys in a [`StorageMap`](../common-collections/storage_map.md).

Unlike Rust, which can automatically generate a `Hash` implementation via [`#[derive(Hash)]`](https://doc.rust-lang.org/std/hash/trait.Hash.html#derivable), Sway does not currently support deriving trait implementations. The `Hash` trait must therefore always be implemented manually.

## The `Hash` trait

```sway
pub trait Hash {
    fn is_hash_trivial() -> bool;

    fn hash(self, ref mut state: Hasher);
}
```

An implementation consists of two methods:

- `hash` defines how the value writes its bytes into the `Hasher`. This is the only method that affects the resulting hash value.
- `is_hash_trivial` is an optimization hint. It declares whether the value's *in-memory representation* is byte-for-byte identical to the bytes that `hash` writes into the `Hasher`.

### Implementing `hash`

`hash` writes the *hash bytes representation* of the value into the `Hasher`. For aggregates, this is typically done by hashing each field in turn, and for enums by first hashing a discriminator (the tag) and then the payload:

```sway
use std::hash::{Hash, Hasher};

struct Point {
    x: u64,
    y: u64,
}

impl Hash for Point {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        self.x.hash(state);
        self.y.hash(state);
    }
}
```

### Implementing `is_hash_trivial`

When a type is *trivially hashable*, its hash can be computed directly from the raw memory of the value (the `__size_of::<Self>()` bytes at the value's address), without first building an intermediate byte buffer in a `Hasher`. This is exactly what `sha256` and `keccak256` do for trivially hashable types, and it is significantly more gas efficient.

Returning `true` is a **strong guarantee**: an incorrect `true` will produce wrong hashes when a value is hashed via `sha256` or `keccak256`. Returning `false` is **always safe**; it only forgoes the optimization.

> **When in doubt, return `false`.**

## Rules for safely implementing `is_hash_trivial`

Return `true` only if the in-memory representation of the type is byte-for-byte identical to the bytes its `hash` method writes into the `Hasher`. Several subtleties make types that look trivially hashable actually **not** trivially hashable:

- **`u16` and `u32` are never trivially hashable.** They are stored in memory in an eight-byte slot (as a `u64`), but their hash bytes representation is only two and four bytes, respectively. Any aggregate (struct, tuple, array, ...) containing them is therefore also not trivially hashable.
- **Padding inside aggregates breaks triviality.** `bool`, `u8`, `u16`, and `u32` fields inside a struct or tuple are padded to eight bytes in memory, while `hash` writes them without that padding. An aggregate containing such a field is therefore **not** trivially hashable, even though the field types might be when hashed on their own.
- **Enum tags are stored as `u64`.** The `Hash` implementations in the standard library hash enum tags as `u8`, but the tag is stored as a `u64` in memory.  Enums following that convention are therefore **not** trivially hashable.
- **Collections depend on the `new_hashing` feature.** `Bytes`, `Vec`, `raw_slice`, `str`, `str[N]`, arrays, and any aggregate containing them can be trivially hashable or not depending on the [`new_hashing`](https://github.com/FuelLabs/sway/issues/7256) experimental feature. When `new_hashing` is enabled, collections prefix their content with their length, so their hash bytes representation no longer matches their in-memory representation, making them **not** trivially hashable.

A type is trivially hashable when it is a fixed-size type with no padding whose `hash` method writes exactly its in-memory bytes. This includes `u64`, `b256`, `u256`, `bool`, and `()`, as well as structs and tuples whose fields are all
themselves trivially hashable and word-aligned (e.g. only `u64`, `b256`, `u256`).

## Examples

### A trivially hashable struct

A struct whose fields are all word-aligned and trivially hashable, with no padding, is trivially hashable:

```sway
use std::hash::{Hash, Hasher};

struct Stats {
    strength: u64,
    agility: u64,
}

impl Hash for Stats {
    fn is_hash_trivial() -> bool {
        // Two `u64` fields, no padding: the in-memory bytes are exactly
        // the bytes written by `hash`.
        true
    }

    fn hash(self, ref mut state: Hasher) {
        self.strength.hash(state);
        self.agility.hash(state);
    }
}
```

### A struct that is not trivially hashable

Padded fields (`bool`) and dynamically sized fields (`str`) make a struct not trivially hashable:

```sway
use std::hash::{Hash, Hasher};

struct Account {
    id: u64,
    active: bool, // Padded to eight bytes in memory.
    name: str, // Dynamically sized.
}

impl Hash for Account {
    fn is_hash_trivial() -> bool {
        // `active` is padded to eight bytes in memory, and `name` is
        // dynamically sized, so the in-memory representation does not
        // match the hash bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        self.id.hash(state);
        self.active.hash(state);
        self.name.hash(state);
    }
}
```

### Enums

Following the standard library convention of hashing the tag as a `u8` makes an enum **not** trivially hashable, because the tag is stored as a `u64` in memory:

```sway
use std::hash::{Hash, Hasher};

enum Shape {
    Circle: u64,
    Square: u64,
}

impl Hash for Shape {
    fn is_hash_trivial() -> bool {
        // The tag is hashed as a `u8` but stored as a `u64` in memory.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        match self {
            Shape::Circle(radius) => {
                0_u8.hash(state);
                radius.hash(state);
            },
            Shape::Square(side) => {
                1_u8.hash(state);
                side.hash(state);
            },
        }
    }
}
```

An enum can be made trivially hashable by hashing the tag as a `u64` (matching its in-memory representation). The simplest safe case is a *tag-only* enum, i.e.  an enum whose variants are all unit (zero-sized):

```sway
use std::hash::{Hash, Hasher};

enum Location {
    Earth: (),
    Mars: (),
}

impl Hash for Location {
    fn is_hash_trivial() -> bool {
        // The enum consists only of its tag, hashed as a `u64`, which
        // matches its in-memory representation.
        true
    }

    fn hash(self, ref mut state: Hasher) {
        match self {
            Location::Earth => 0_u64.hash(state),
            Location::Mars => 1_u64.hash(state),
        }
    }
}
```
