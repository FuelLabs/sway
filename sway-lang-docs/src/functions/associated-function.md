# Associated Functions

_Associated functions_ are very similar to _methods_, in that they are also defined in the context of a struct or enum, but they do not actually use any of the data in the struct and as a result do not take _self_ as a parameter. Associated functions could be standalone functions, but they are included in a specific type for organizational or semantic reasons.

To declare methods and associated functions for a struct or enum, use an _impl block_. Here, `impl` stands for implementation.

```sway
script;

struct Foo {
    bar: u64,
    baz: bool,
}

impl Foo {
    // this is an _associated function_, since it does not take `self` as a parameter.
    fn new_foo(number: u64, boolean: bool) -> Foo {
        Foo {
            bar: number,
            baz: boolean,
        }
    }
}

fn main() {
    let foo = ~Foo::new_foo(42, true);
}
```

Note the syntax of the associated function call: `~Foo::new_foo(42, true);`. This bit of syntax is unique to Sway: when referring to a type directly, you preface the type with a tilde (`~`). To call an associated function, refer to the type and then the function name.
