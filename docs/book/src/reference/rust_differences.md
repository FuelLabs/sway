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

In Sway, enums are simplified. Enums variants must all specify exactly one type. This type represents their interior data. This is actually isomorphic to what Rust offers, just with a different syntax. I'll now rewrite the above enum but with Sway syntax:

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
