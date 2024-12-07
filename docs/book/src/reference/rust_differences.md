# Differences From Rust

Sway shares a lot with Rust, especially its syntax. Because they are so similar, you may be surprised or caught off guard when they differ. This page serves to outline, from a high level, some of the syntactic _gotchas_ that you may encounter.

## Enum Variant Syntax

In Rust, enums generally take one of three forms: _unit_ variants, which have no inner data, _struct_ variants, which contain named fields, and _tuple_ variants, which contain within them a tuple of data. If you are unfamiliar with these terms, this is what they look like:

```rust,ignore
// note to those skimming the docs: this is Rust syntax! Not Sway! Don't copy/paste this into a Sway program.

enum Foo {
    UnitVariant,
    TupleVariant(u32, u64, bool),
    StructVariant {
        field_one: bool,
        field_two: bool
    }
}
```

In Sway, enums are simplified. Enums variants must all specify exactly one type. This type represents their interior data. This is actually isomorphic to what Rust offers, but with a different syntax. You can see the above enum but with Sway syntax below:

```sway
// This is equivalent Sway syntax for the above Rust enum.
enum Foo {
    UnitVariant: (),
    TupleVariant: (u32, u64, bool),
    StructVariant: MyStruct,
}

struct MyStruct {
    field_one: bool,
    field_two: bool,
}
```

## Memory Allocation

In Rust, the borrow checker implements Rust's [ownership system](https://doc.rust-lang.org/1.8.0/book/ownership.html)

In Sway, there is no borrow checker.  This means there is no concept of ownership, borrowing, or lifetimes.  Instead, objects are copied and moved similar to C++.  Also Sway does not have any destructors nor `Drop` traits.  This means allocated memory lives for the entire transaction and is not deallocated until the end of the transaction.  A transaction may allocate up to [64 MB](https://github.com/FuelLabs/fuel-vm/blob/a80f82ed7c793763de6a73ca72d946b311b0fd0b/fuel-vm/src/consts.rs#L26) of memory.
