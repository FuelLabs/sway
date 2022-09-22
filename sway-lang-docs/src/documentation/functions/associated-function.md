# Associated Functions

_Associated functions_ are very similar to _methods_ in that they are also defined in the context of a struct or enum, but they do not use any of the data in the struct and as a result do not take _self_ as a parameter. 

Associated functions could be standalone functions, but they are included in a specific type for organizational or semantic reasons.

### Decleration

To declare an associated functions for a struct or enum, use an _impl block_. Here, `impl` stands for implementation.

```sway
struct Foo {
    bar: u64,
    baz: bool,
}

impl Foo {
    // this is an associated function because it does not take `self` as a parameter
    fn new(number: u64, boolean: bool) -> Self {
        Self {
            bar: number,
            baz: boolean,
        }
    }
}
```

### Usage

The syntax to call an associated function is unique to Sway. When referring to a type directly you preface the type with a tilde `~`.

```sway
fn main() {
    let foo = ~Foo::new(42, true);
}
```
